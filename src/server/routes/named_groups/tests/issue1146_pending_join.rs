use super::*;

async fn route_join(
    state: &Arc<AppState>,
    link: String,
    group_id: &str,
    owner_pin: &str,
) -> Result<(String, NamedGroupMetadataEvent)> {
    let response = join_group_via_invite(
        State(Arc::clone(state)),
        Json(JoinGroupRequest {
            invite: link,
            display_name: None,
            mode: Some("home".to_string()),
            expected_owner_user_id: Some(owner_pin.to_string()),
        }),
    )
    .await
    .into_response();
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let body: serde_json::Value = serde_json::from_slice(&body)?;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["join_state"], "pending_authority_commit");
    let member = hex::encode(state.agent.agent_id().as_bytes());
    let attempts = state
        .pending_join_attempts
        .lock()
        .expect("attempt registry");
    let attempt = attempts
        .get(&join_result_key(group_id, &member))
        .expect("route installed an attempt");
    Ok((
        attempt.attempt_id.clone(),
        attempt
            .stored_resend
            .as_ref()
            .expect("stored join")
            .event
            .clone(),
    ))
}

/// Deliver the authority's real result without a network round trip: inline
/// the exact staged Welcome bytes, leaving the signed commit/mandate unchanged.
async fn deliver_result(
    authority: &Arc<AppState>,
    joiner: &Arc<AppState>,
    group_id: &str,
    attempt_id: &str,
) {
    let member = hex::encode(joiner.agent.agent_id().as_bytes());
    let pending = authority
        .pending_join_results
        .read()
        .await
        .get(&join_result_key(group_id, &member))
        .cloned()
        .expect("authority staged a result");
    let mut event = pending.event;
    let NamedGroupMetadataEvent::MemberAdded {
        treekem_welcome_b64,
        welcome_ref,
        ..
    } = &mut event
    else {
        panic!("authority staged a MemberAdded");
    };
    let reference = welcome_ref.take().expect("real staged Welcome reference");
    let welcomes = authority.pending_welcomes.read().await;
    let welcome = welcomes.get(&reference.welcome_id).expect("staged Welcome");
    assert_eq!(welcome.group_id, group_id);
    assert_eq!(welcome.joiner_agent, member);
    *treekem_welcome_b64 = Some(BASE64.encode(&welcome.bytes));
    drop(welcomes);
    handle_join_result_message_bound(
        joiner,
        &authority.agent.agent_id(),
        true,
        JoinResultMessage::Result {
            event: Box::new(event),
            chain: Vec::new(),
            head_attestation: pending.head_attestation.map(Box::new),
            roster_certificates_b64: Vec::new(),
            intervening_events: Vec::new(),
        },
        Some(attempt_id),
    )
    .await;
}

/// #1146: another member's pre-Welcome state-only apply must not promote
/// an unseated joiner's stub to durable state or prevent refusal cleanup.
#[tokio::test]
async fn issue1146_other_member_added_keeps_stub_pending_until_refusal_and_retry() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let owner_seed = [0x46; 32];
    let mut states = Vec::new();
    for name in ["authority", "b", "d"] {
        let data_dir = dir.path().join(name);
        tokio::fs::create_dir_all(&data_dir).await?;
        let agent = Arc::new(
            Agent::builder()
                .with_machine_key(data_dir.join("machine.key"))
                .with_agent_key(x0x::identity::AgentKeypair::generate()?)
                .with_agent_cert_path(data_dir.join("agent.cert"))
                .with_user_key(x0x::identity::UserKeypair::from_seed(&owner_seed)?)
                .with_peer_cache_disabled()
                .with_contact_store_path(data_dir.join("contacts.json"))
                .with_network_config(isolated_loopback_config("treekem"))
                .build()
                .await?,
        );
        states.push(secure_endpoint_test_state_at(&data_dir, agent).await?);
    }
    let authority = &states[0];
    let b = &states[1];
    let d = &states[2];
    super::super::super::home::provision_home(authority).await;
    let owner = authority.agent.identity().user_keypair().expect("owner");
    let owner_pin = hex::encode(owner.user_id().as_bytes());
    let (_, home) = super::super::super::home::find_home(authority, &owner.user_id())
        .await
        .expect("provisioned Home");
    let group_key = home.mls_group_id;
    let (invite, link) = mint_invite_transaction(
        authority,
        &group_key,
        3_600,
        None,
        x0x::groups::InviteOrigin::Explicit,
        true,
    )
    .await
    .map_err(|e| anyhow::anyhow!("mint invite: {e:?}"))?;
    assert!(invite.intended_joiner.is_none(), "one unaddressed invite");
    let group_id = authority.named_groups.read().await[&group_key]
        .stable_group_id()
        .to_string();
    let b_hex = hex::encode(b.agent.agent_id().as_bytes());
    let d_hex = hex::encode(d.agent.agent_id().as_bytes());
    let d_result_key = join_result_key(&group_id, &d_hex);

    // D is genuinely pending BEFORE B consumes the same one-time invite.
    let (d_attempt, d_join) = route_join(d, link.clone(), &group_id, &owner_pin).await?;
    let d_tasks: Vec<_> = ATTEMPT_TASK_HANDLES_FOR_TEST
        .lock()
        .expect("task handles")
        .iter()
        .filter(|(id, _)| id == &d_attempt)
        .map(|(_, handle)| handle.clone())
        .collect();
    assert_eq!(d_tasks.len(), 5, "all route-owned tasks captured");
    let (listener_token, listener_handle) = {
        let listeners = d.group_metadata_tasks.read().await;
        let listener = listeners.get(&group_key).expect("route listener");
        (listener.token, listener.handle.abort_handle())
    };
    assert_eq!(
        d.pending_join_attempts.lock().expect("attempts")[&d_result_key].listener_token,
        Some(listener_token)
    );
    assert!(d
        .expected_join_result_inviters
        .lock()
        .expect("pins")
        .contains_key(&d_result_key));
    assert!(d
        .pending_join_stubs
        .lock()
        .expect("stubs")
        .contains(&group_key));
    assert!(
        !load_named_groups_merged(&d.named_groups_path, &d.home_suite_groups_path)
            .await?
            .contains_key(&group_key)
    );

    let (b_attempt, b_join) = route_join(b, link, &group_id, &owner_pin).await?;
    assert!(
        apply_named_group_metadata_event(authority, b_join, b.agent.agent_id(), true, None)
            .await
            .accepted,
        "B consumed the invite through authority apply"
    );
    let b_result = authority
        .pending_join_results
        .read()
        .await
        .get(&join_result_key(&group_id, &b_hex))
        .cloned()
        .expect("B's real authority result");
    assert!(
        matches!(&b_result.event, NamedGroupMetadataEvent::MemberAdded {
        agent_id, treekem_commit_b64: Some(_), owner_mandate: Some(_), commit: Some(_), ..
    } if agent_id == &b_hex)
    );
    deliver_result(authority, b, &group_id, &b_attempt).await;
    assert!(b.treekem_groups.read().await.contains_key(&group_key));
    assert!(b.named_groups.read().await[&group_key].has_active_member(&b_hex));

    // The real, signed B update reaches unseated D BEFORE its refusal.
    assert!(
        apply_named_group_metadata_event(
            d,
            b_result.event.clone(),
            authority.agent.agent_id(),
            true,
            None
        )
        .await
        .accepted,
        "B's update applies through pre-Welcome persistence"
    );
    {
        let groups = d.named_groups.read().await;
        let stub = &groups[&group_key];
        assert!(stub.has_active_member(&b_hex));
        assert!(
            !stub.members_v2.contains_key(&d_hex),
            "D has no seat of its own"
        );
        assert_eq!(
            stub.state_revision,
            named_group_metadata_event_commit(&b_result.event)
                .expect("signed commit")
                .revision
        );
    }
    assert!(!d.treekem_groups.read().await.contains_key(&group_key));
    assert!(
        d.pending_join_stubs
            .lock()
            .expect("stubs")
            .contains(&group_key),
        "#1146: persisting B's seat must keep D's pending marker"
    );
    assert!(
        !load_named_groups_merged(&d.named_groups_path, &d.home_suite_groups_path)
            .await?
            .contains_key(&group_key),
        "D's pending stub must remain excluded from disk"
    );
    assert_eq!(
        d.pending_join_attempts.lock().expect("attempts")[&d_result_key].attempt_id,
        d_attempt
    );

    assert!(
        !apply_named_group_metadata_event(authority, d_join, d.agent.agent_id(), true, None)
            .await
            .accepted,
        "D's reuse is refused by authority apply"
    );
    // As in wp_b_t10, production FetchRequest signs the staged refusal;
    // transport cannot deliver it, so inject that exact served receipt.
    handle_join_result_message(
        authority,
        &d.agent.agent_id(),
        true,
        JoinResultMessage::FetchRequest {
            group_id: group_id.clone(),
            member_agent_id: d_hex.clone(),
            from_revision: None,
            base_state_hash: None,
            accepts_refusal: true,
            accepts_control_blob_ref: false,
            attempt_id: Some(d_attempt.clone()),
        },
    )
    .await;
    let receipt = {
        let refusals = authority.pending_join_refusals.lock().expect("refusals");
        let pending = refusals
            .get(&join_refusal_refusal_key(&group_id, &d_hex, &d_attempt))
            .expect("D's bound refusal");
        let mut receipt = pending.receipt.clone();
        receipt.signature_b64 = pending
            .cached_signature_b64
            .clone()
            .expect("fetch signed refusal");
        receipt
    };
    assert_eq!(receipt.reason, JoinRefusalReason::InviteSecretConsumed);
    handle_join_result_message(
        d,
        &authority.agent.agent_id(),
        true,
        JoinResultMessage::Refused {
            receipt: Box::new(receipt),
        },
    )
    .await;
    let outcome = d
        .last_join_outcomes
        .lock()
        .expect("outcomes")
        .get(&group_key)
        .cloned()
        .expect("terminal refusal");
    assert_eq!(outcome.outcome, "refused");
    assert_eq!(outcome.reason, Some("invite_secret_consumed"));
    assert!(
        !d.named_groups.read().await.contains_key(&group_key),
        "D's stub removed"
    );
    assert!(!d
        .pending_join_stubs
        .lock()
        .expect("stubs")
        .contains(&group_key));
    assert!(!d
        .pending_join_attempts
        .lock()
        .expect("attempts")
        .contains_key(&d_result_key));
    assert!(!d
        .expected_join_result_inviters
        .lock()
        .expect("pins")
        .contains_key(&d_result_key));
    assert!(
        !d.group_metadata_tasks.read().await.contains_key(&group_key),
        "owned listener removed"
    );
    assert!(!d.mls_groups.read().await.contains_key(&group_key));
    assert!(!d.treekem_groups.read().await.contains_key(&group_key));
    for _ in 0..40 {
        if d_tasks.iter().all(|h| h.is_finished()) && listener_handle.is_finished() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert!(
        d_tasks.iter().all(|h| h.is_finished()),
        "all D attempt tasks stopped"
    );
    assert!(listener_handle.is_finished(), "D's owned listener stopped");
    assert!(
        !load_named_groups_merged(&d.named_groups_path, &d.home_suite_groups_path)
            .await?
            .contains_key(&group_key)
    );
    {
        let groups = authority.named_groups.read().await;
        assert!(groups[&group_key].has_active_member(&b_hex));
        assert!(!groups[&group_key].has_active_member(&d_hex));
    }
    let preserved = authority
        .pending_join_results
        .read()
        .await
        .get(&join_result_key(&group_id, &b_hex))
        .cloned()
        .expect("B's result preserved");
    assert_eq!(
        serde_json::to_value(&preserved.event)?,
        serde_json::to_value(&b_result.event)?
    );
    assert!(b.named_groups.read().await[&group_key].has_active_member(&b_hex));
    assert!(b.treekem_groups.read().await.contains_key(&group_key));

    // A fresh addressed invite must install a NEW attempt, not hit the
    // already-joined shortcut left behind by the prematurely durable stub.
    let (fresh_invite, fresh_link) = mint_invite_transaction(
        authority,
        &group_key,
        3_600,
        Some(d.agent.agent_id()),
        x0x::groups::InviteOrigin::Explicit,
        true,
    )
    .await
    .map_err(|e| anyhow::anyhow!("mint addressed invite: {e:?}"))?;
    assert!(fresh_invite.intended_joiner.is_some());
    let (new_attempt, new_join) = route_join(d, fresh_link, &group_id, &owner_pin).await?;
    assert_ne!(new_attempt, d_attempt);
    assert!(d
        .pending_join_stubs
        .lock()
        .expect("stubs")
        .contains(&group_key));
    assert!(!d
        .last_join_outcomes
        .lock()
        .expect("outcomes")
        .contains_key(&group_key));
    assert!(
        apply_named_group_metadata_event(authority, new_join, d.agent.agent_id(), true, None)
            .await
            .accepted,
        "fresh addressed invite seats D on the authority"
    );
    deliver_result(authority, d, &group_id, &new_attempt).await;
    assert!(d.named_groups.read().await[&group_key].has_active_member(&d_hex));
    assert!(
        d.treekem_groups.read().await.contains_key(&group_key),
        "D consumed its own Welcome"
    );
    assert!(!d
        .pending_join_stubs
        .lock()
        .expect("stubs")
        .contains(&group_key));
    assert!(!d
        .pending_join_attempts
        .lock()
        .expect("attempts")
        .contains_key(&d_result_key));
    assert!(
        load_named_groups_merged(&d.named_groups_path, &d.home_suite_groups_path).await?
            [&group_key]
            .has_active_member(&d_hex),
        "D's own seat is durable"
    );
    Ok(())
}
