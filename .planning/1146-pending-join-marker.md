# Issue 1146: Preserve Pending Join State Through Roster Catch-Up

Date: 2026-10-01. Status: deterministic Linux red confirmed; prevention fix in progress.
Issue: [x0x #1146](https://github.com/saorsa-labs/x0x/issues/1146).
Base: `eb4c6b7e690aaa892a25d4d18280b257586518d8` (upstream main, 0.46.0).
Branch: `fix/1146-pending-join-marker`.

## Scope And Authority

Owner approved a bounded prevention fix and an independent fixture, not a new
protocol or a production deployment. The working invariant: another member's
signed addition may advance an unseated joiner's roster, but must not confirm
that joiner's own membership or make its temporary record durable.

Read: AGENTS, Accepted ADR 0059 (authenticated invite/seating provenance),
Accepted ADR 0106 (ordinary verified intervening-event application), production
persist/finalize/join routes, and the nearest consumed-invite and back-to-back
TreeKEM tests. Accepted records remain unchanged. This restores the existing
pending-state/seat distinction; no new architectural decision is proposed.

Allowed: one focused in-crate test module, its registration, a minimal
`persist_named_group_info_inner` correction, and this evidence record.
Forbidden: crypto/wire/storage-format changes, dependency/lock changes, gate or
test-wrapper edits, wider fixture permissions, altered existing expectations,
upstream PR, merge, release, deploy, installed-daemon/key access or paid API calls.
Previously stranded durable rows need separate recovery work; prevention does
not automatically migrate them.

## Regression And Fix Contract

The fixture mints one real unaddressed Home/TreeKEM invite. D installs a real
pending route attempt; B consumes the invite through authority apply. B's actual
signed MemberAdded reaches D before D's refusal. D's revision must advance while
its own seat/key remain absent, its pending marker remains, and disk excludes
its stub. A production-signed refusal then removes the stub and attempt-owned
resources. B's authority result and seat remain. A fresh addressed invite starts
a new D attempt and its real authority result seats D.

Deliveries in the test are explicit in-process handler calls, not a native
network-delivery proof. Exact staged Welcome bytes are inlined for handler
testing, without changing the signed commit or mandate. No synthetic new crypto.
The existing delayed-refusal/seat-wins tests remain required controls.

Minimal proposed correction: remove a pending marker during persistence only
when the validated stored roster includes the local agent as an active member.
Retain current lock ordering and rollback. Investigate any other pending-marker
removal/journal exception before claiming route-wide prevention; do not silently
expand this into recovery or lifecycle redesign.

## Verification

This host is macOS. Agent tests must not run here outside the declared Linux
namespace wrapper. Local `cargo check --locked --tests --all-features` and format
are compilation/static support, not runtime evidence.

Declared focused Linux command:

```sh
python3 scripts/dev/test-isolated.py nextest --locked --all-features --lib -- -E 'test(issue1146_other_member_added_keeps_stub_pending_until_refusal_and_retry)' --test-threads 1
```

Existing CI invokes `scripts/ci/nextest-isolated.sh` for the full suite, including
this in-crate module. Keep that workflow unchanged. Red checkpoint is upstream
code plus test only; expected assertion is that persisting B's seat must retain
D's pending marker. Green requires the same fixture unchanged after the fix,
plus ordinary joins, refusal, delayed-result and seat-wins controls.

Jim approved intended commits/pushes and a draft **CI-only PR into his own public
fork**, not an upstream contribution PR. He subsequently approved advancing
the fork's main to upstream by fast-forward (no unique main commits lost). The
fork is public; standard hosted Linux/macOS CI has no paid overage approval.
CI arbiter: [CI-only fork draft PR #8](https://github.com/JimCollinson/x0x/pull/8).
No `.gsd/gate.sh` exists; do not add one here.

Craft Review per meaningful unit, independent clean-context verification of
the patch, and adversarial report at milestone. No readiness claim with missing
CI or hidden/uncommitted scripts. Preserve exact red/green job URLs and failures;
do not dismiss an unrelated failure as pre-existing without base evidence.

Unattended until this bounded checkpoint; no new spend, publishing, upstream PR
or acceptance of architecture decisions. Record needed escalations rather than
rewriting verification machinery.

## Evidence

- Test authored: `issue1146_other_member_added_keeps_stub_pending_until_refusal_and_retry`.
- Local test compilation: `cargo check --locked --tests --all-features` passed.
- Local format: `cargo fmt --all -- --check` passed.
- Local whitespace: `git diff --check` passed.
- No test fixture runtime executed on macOS.
- Fresh test-only Craft Review: no anchored findings; source-only, no runtime
  correctness claim. Coordinator verified the intended three-file change set.
- Linux red confirmed at test-only commit `7b19430c3b47f37905e3303dad0b093b3029f8d6`,
  [CI Test Suite job](https://github.com/JimCollinson/x0x/actions/runs/36868991405/job/110391600044).
  At 13:53:15 UTC the new regression failed in 0.814 seconds at
  `issue1146_pending_join.rs:231`: "persisting B's seat must keep D's pending marker".
  The ordinary pre-Welcome apply had succeeded and D had no local seat/TreeKEM
  group; the precise marker-clearing failure is now deterministically exercised.
  Full suite fail-fast: 3,067 passed, one new regression failed, 326 skipped,
  1,653 remaining tests not run. This is baseline red, not a complete suite pass.
- The fix changes only the existing pending-marker removal predicate in
  `persist_named_group_info_inner`. The regression file remains unchanged from
  the red checkpoint. No protocol, lock, serializer, rollback, or test config edit.
- Prevention is bounded to newly pending pre-Welcome records with no local crypto
  snapshot. Old durable residue/recovery and pending records with leftover crypto
  state remain outside this patch's verified claim.
- Fresh prevention-patch Craft Review: pass, no anchored findings. It checked
  the predicate/comment against adjacent membership and rollback code, not
  runtime correctness. Coordinator inspected the exact two-file diff and verified
  the regression is byte-for-byte unchanged from the red commit.
- Local pinned compilation: `rustup run 1.95.0 cargo check --locked --tests --all-features`
  passed after the prevention change; formatting and whitespace passed.
- Green CI and final correctness/clean-context/adversarial reviews: pending.
- Local Clippy attempts found `result_large_err` in unchanged `network.rs:1752`;
  they are not used as readiness evidence and no warning was suppressed. Linux
  CI Clippy passed on the test-only branch. The local cargo/clippy selection
  needs investigation before making a local lint-green claim.
  An explicit cargo-clippy invocation also failed with incompatible-compiler
  metadata diagnostics in the shared local target; no clean or environment
  change was made to hide the error. These local lint attempts remain failures,
  not an asserted pre-existing or environmental exclusion. CI is the arbiter.
- Fork main's history-sync ADR job failed on an Accepted ADR change across the
  large upstream fast-forward: [base push evidence](https://github.com/JimCollinson/x0x/actions/runs/36868196676).
  No ADR was changed by this slice; that history-sync check is not green or fixed
  here. The PR compares only the bounded changes against synced main.
