//! Property-based tests for groups.
#![allow(clippy::unwrap_used)]

use proptest::prelude::*;
use std::collections::BTreeMap;
use x0x::groups::{
    card::AgentCard, enforce_last_admin_invariant, invite::SignedInvite, ApplyError, GroupInfo,
    GroupMember, GroupMemberState, GroupRole,
};
use x0x::identity::AgentId;

fn agent(bytes: [u8; 32]) -> AgentId {
    AgentId(bytes)
}

#[derive(Debug, Clone)]
struct RosterMemberSpec {
    role: GroupRole,
    state: GroupMemberState,
}

fn member_spec(role: GroupRole, state: GroupMemberState) -> RosterMemberSpec {
    RosterMemberSpec { role, state }
}

fn arb_member_spec() -> impl Strategy<Value = RosterMemberSpec> {
    (
        prop_oneof![
            Just(GroupRole::Owner),
            Just(GroupRole::Admin),
            Just(GroupRole::Moderator),
            Just(GroupRole::Member),
            Just(GroupRole::Guest),
        ],
        prop_oneof![
            Just(GroupMemberState::Active),
            Just(GroupMemberState::Pending),
            Just(GroupMemberState::Removed),
            Just(GroupMemberState::Banned),
        ],
    )
        .prop_map(|(role, state)| member_spec(role, state))
}

fn arb_last_admin_case() -> impl Strategy<Value = (Vec<RosterMemberSpec>, bool)> {
    prop_oneof![
        // Zero-admin live rosters.
        Just((Vec::new(), false)),
        Just((
            vec![member_spec(GroupRole::Member, GroupMemberState::Active)],
            false
        )),
        Just((
            vec![member_spec(GroupRole::Moderator, GroupMemberState::Active)],
            false
        )),
        Just((
            vec![member_spec(GroupRole::Guest, GroupMemberState::Active)],
            false
        )),
        // Exactly-one-admin live boundaries.
        Just((
            vec![member_spec(GroupRole::Admin, GroupMemberState::Active)],
            false
        )),
        Just((
            vec![member_spec(GroupRole::Owner, GroupMemberState::Active)],
            false
        )),
        // Admin-rank members in non-active states must not count.
        Just((
            vec![member_spec(GroupRole::Admin, GroupMemberState::Banned)],
            false
        )),
        Just((
            vec![member_spec(GroupRole::Owner, GroupMemberState::Banned)],
            false
        )),
        Just((
            vec![member_spec(GroupRole::Admin, GroupMemberState::Removed)],
            false
        )),
        Just((
            vec![member_spec(GroupRole::Owner, GroupMemberState::Pending)],
            false
        )),
        // Withdrawal is the explicit exit valve even with no active admins.
        Just((Vec::new(), true)),
        Just((
            vec![member_spec(GroupRole::Admin, GroupMemberState::Banned)],
            true
        )),
        // Mixed arbitrary rosters keep shrinking useful while still exploring.
        (
            prop::collection::vec(arb_member_spec(), 0..16),
            any::<bool>()
        ),
    ]
}

fn roster_from_specs(specs: &[RosterMemberSpec]) -> BTreeMap<String, GroupMember> {
    specs
        .iter()
        .enumerate()
        .map(|(idx, spec)| {
            let agent_id = format!("{idx:064x}");
            (
                agent_id.clone(),
                GroupMember {
                    agent_id,
                    user_id: None,
                    role: spec.role,
                    state: spec.state,
                    display_name: None,
                    joined_at: idx as u64,
                    updated_at: idx as u64,
                    added_by: None,
                    removed_by: None,
                    kem_public_key_b64: None,
                    treekem_key_package_b64: None,
                },
            )
        })
        .collect()
}

fn independent_last_admin_oracle(specs: &[RosterMemberSpec], withdrawn: bool) -> bool {
    withdrawn
        || specs.iter().any(|spec| {
            matches!(spec.state, GroupMemberState::Active)
                && matches!(spec.role, GroupRole::Admin | GroupRole::Owner)
        })
}

proptest! {
    #[test]
    fn invite_link_roundtrip(
        group_id_bytes in prop::array::uniform16(any::<u8>()),
        group_name in prop::string::string_regex("[a-zA-Z0-9 -]{1,32}").unwrap(),
        inviter_bytes in prop::array::uniform32(any::<u8>()),
        expiry_secs in 0u64..1_000_000,
    ) {
        let inviter = agent(inviter_bytes);
        let invite = SignedInvite::new(
            hex::encode(group_id_bytes),
            group_name.clone(),
            &inviter,
            expiry_secs,
        );

        let parsed = SignedInvite::from_link(&invite.to_link());
        prop_assert!(parsed.is_ok());
        let parsed = parsed.unwrap();

        prop_assert_eq!(parsed.group_id, invite.group_id);
        prop_assert_eq!(parsed.group_name, group_name);
        prop_assert_eq!(parsed.inviter, invite.inviter);
        prop_assert_eq!(parsed.invite_secret, invite.invite_secret);
        prop_assert_eq!(parsed.expires_at, invite.expires_at);
    }

    #[test]
    fn signable_bytes_deterministic(
        group_id_bytes in prop::array::uniform16(any::<u8>()),
        group_name in prop::string::string_regex("[a-zA-Z0-9 -]{1,32}").unwrap(),
        inviter_bytes in prop::array::uniform32(any::<u8>()),
        expiry_secs in 0u64..1_000_000,
    ) {
        let invite = SignedInvite::new(
            hex::encode(group_id_bytes),
            group_name,
            &agent(inviter_bytes),
            expiry_secs,
        );
        prop_assert_eq!(invite.signable_bytes(), invite.signable_bytes());
    }

    #[test]
    fn general_chat_topic_uses_general_room(
        name in prop::string::string_regex("[a-zA-Z]{1,16}").unwrap(),
        description in prop::string::string_regex("[a-zA-Z0-9 ]{0,32}").unwrap(),
        creator_bytes in prop::array::uniform32(any::<u8>()),
        group_id_bytes in prop::array::uniform16(any::<u8>()),
    ) {
        let info = GroupInfo::new(
            name,
            description,
            agent(creator_bytes),
            hex::encode(group_id_bytes),
        );
        let topic = info.general_chat_topic();
        prop_assert!(topic.starts_with("x0x.group."));
        prop_assert!(topic.ends_with("/general"));
    }

    #[test]
    fn display_name_fallback_is_non_empty(
        name in prop::string::string_regex("[a-zA-Z]{1,16}").unwrap(),
        creator_bytes in prop::array::uniform32(any::<u8>()),
        member_bytes in prop::array::uniform32(any::<u8>()),
        group_id_bytes in prop::array::uniform16(any::<u8>()),
    ) {
        let info = GroupInfo::new(
            name,
            String::new(),
            agent(creator_bytes),
            hex::encode(group_id_bytes),
        );
        let fallback = info.display_name(&hex::encode(member_bytes));
        prop_assert!(!fallback.is_empty());
    }

    #[test]
    fn agent_card_link_roundtrip(
        agent_bytes in prop::array::uniform32(any::<u8>()),
        machine_bytes in prop::array::uniform32(any::<u8>()),
        display_name in prop::string::string_regex("[a-zA-Z0-9_-]{1,16}").unwrap(),
    ) {
        let agent_id = agent(agent_bytes);
        let machine_id = hex::encode(machine_bytes);
        let card = AgentCard::new(display_name.clone(), &agent_id, &machine_id);

        let parsed = AgentCard::from_link(&card.to_link());
        prop_assert!(parsed.is_ok());
        let parsed = parsed.unwrap();

        prop_assert!(parsed.short_display().contains(&parsed.display_name));
        prop_assert_eq!(&parsed.agent_id, &hex::encode(agent_bytes));
        prop_assert_eq!(&parsed.machine_id, &machine_id);
        prop_assert_eq!(parsed.display_name, display_name);
    }

    #[test]
    fn last_admin_invariant_matches_independent_oracle(case in arb_last_admin_case()) {
        let (specs, withdrawn) = case;
        let members = roster_from_specs(&specs);
        let result = enforce_last_admin_invariant(&members, withdrawn);
        let expected_ok = independent_last_admin_oracle(&specs, withdrawn);

        prop_assert_eq!(result.is_ok(), expected_ok);
        if !expected_ok {
            prop_assert!(matches!(result, Err(ApplyError::Invariant(_))));
        }
    }
}
