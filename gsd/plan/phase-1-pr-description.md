# ADR-0016 Phase 1 PR description draft

## Design decisions worth your eye

### Receive-path authority: creator gates removed; pre-existing sender-binding left intact

ADR-0016 decides authority by committed-roster role in `validate_apply`. On the gossip-apply path we deleted every creator-identity authority gate (`sender == creator`) and replaced it with the role check (`at_least(Admin)`) plus signed-commit validation. The receive arms also carry a pre-existing `actor == sender_hex` sender-binding — the daemon delivers membership events directly to active members, so the actor is the sender on the normal path. We left that intact, ANDed with the role authority: it is an additional anti-spoofing restriction, not the authority decision and not an escalation vector (authority is the signed commit + Admin role). Whether to relax it for full relay-tolerance is a maintainer call on the delivery model — flagged here, out of ADR-0016 scope.

### Reserved roles: enforced at authoring, accepted on apply (deliberate)

Role assignment accepts exactly `admin` and `member`. The **role-assignment REST API** rejects `owner`, `moderator`, and `guest` with explicit errors (per ADR-0016: reserved, non-assignable). The **signed gossip-apply path** accepts any validly-signed role label *except* `Owner`. This split is deliberate, and consistent with the ADR placing role-vocabulary enforcement on the *assignment API* (apply-side enforcement is reserved for the last-admin invariant):

- **No authority is at stake.** The only authority threshold in the code is `role.at_least(Admin)` (ranks: Owner 4, Admin 3, Moderator 2, Member 1, Guest 0). `Moderator` and `Guest` fall below `Admin` and appear in no privilege check — an applied reserved role is a member-level participant, nothing more. `Owner` is the one reserved role that *is* admin-equivalent, so it remains rejected on apply.
- **Only an admin can author a role-update**, and an admin can already add anyone as a member directly — so a signed commit carrying `Moderator`/`Guest` grants no capability its author didn't already have.
- **Apply must accept validly-signed peer commits** — including from older/legacy daemons — or replicas diverge and the group forks across versions. Role-vocabulary restrictions therefore belong at the point of *assignment* (authoring), not *receipt* (apply); enforcing them on apply would break byte-for-byte legacy replay and fork live state, for no security gain.

This matches the "Admin is root" model: a non-admin label carries no authority. The membership tests assert this accept-on-apply behaviour intentionally.
