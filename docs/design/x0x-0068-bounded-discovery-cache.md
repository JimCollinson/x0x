# X0X-0068 — Bounded discovery cache + telemetry

**Status:** Ready for execution in a fresh session (zero prior context required)
**Layer:** 1 of 6 in [SOTA-Borrow Phase 2 plan](sota-borrow-phase-2-fleet-survival.md)
**Repos:** `saorsa-gossip` (pubsub crate) + `x0x` (diagnostics)
**Estimated effort:** 1-2 days
**Dependencies:** None — ships first
**Acceptance:** 4h aggregate Phase A soak ≥ 98% with `drop_full=0` and
  non-climbing `max_pp_to`

---

## 0. Read this first

You are picking up the first ticket in a six-ticket portfolio to harden
x0x's broad-launch fleet behaviour. **Do not assume prior conversation
context** — everything you need is here or in linked files.

### What x0x is (one paragraph)

x0x is an agent-to-agent gossip network for AI systems. It pairs
`ant-quic` (QUIC transport with NAT traversal + post-quantum crypto)
with `saorsa-gossip` (epidemic broadcast / pub-sub). The daemon is
`x0xd`, deployed on a 6-node VPS bootstrap mesh (NYC, SFO, Helsinki,
Nuremberg, Singapore, Sydney). The broad-launch gate is the all-pairs
DM matrix (Phase A) under sustained pubsub load.

### What broke

The 2026-05-12 09:34Z 4h certification soak shows the classic "load
grows with state" pattern. Proof:
`proofs/launch-readiness-soak-20260512T093455Z-4h-v0_19_41-rollback-98pct-certification/summary.md`

| Window | Phase A recv/sent | continuous_max_pp_to | drop_full | Verdict |
|---|---|---|---|---|
| 1  | **30/30** | 0    | 0    | **GO** ✓ |
| 5  | 28/28     | 489  | 0    | NO-GO |
| 6  | 27/27     | **1835** | 0 | NO-GO |
| 7  | 28/28     | 1848 | 0    | NO-GO (sup>0.12) |
| 9  | 28/28     | **1878** | 0 | NO-GO |
| 11 | 30/29     | —    | **448** | NO-GO (drop_full>0) |

`continuous_max_pp_to` is the cumulative `republish_per_peer_timeout`
counter from saorsa-gossip — it increments every time a per-peer send
hits the 2500 ms `PER_PEER_REPUBLISH_TIMEOUT`. The fact that it climbs
0 → 1835 across 6 windows on a stable fleet means **the load each peer
must handle is growing during the soak**, even though the workload
(Phase A all-pairs DM, baseline gossip topics) is constant.

The mechanism is anti-entropy of the pubsub message cache: the cache
grows as new gossip messages arrive, anti-entropy must transmit the
delta to lagging peers, cross-region peers (Singapore, Sydney) can't
keep up, they get cooled for 2 minutes, cooling compounds across
topics, eventually `recv_pump.dropped_full` ticks > 0 and the soak
fails outright.

### What we discovered about the cache

`saorsa-gossip/crates/pubsub/src/lib.rs` already bounds the per-topic
message cache **by count**:

```rust
const MAX_CACHE_SIZE: usize = 2_048;                 // line 42
const REPLAY_CACHE_MAX_ENTRIES: usize = 10_000;      // line 53

message_cache: LruCache<MessageIdType, CachedMessage>  // line 1405
replay_cache:  LruCache<[u8; 32], Instant>             // line 1418
```

This bounds count but **not bytes or age**. With observed message sizes
on `x0x.discovery.groups` (11-16 KB per group card), worst-case per-
topic state is `2048 × 16 KB ≈ 32 MB`. Multiplied across active topics,
the mesh anti-entropies ~100 MB+ of state per reconciliation cycle.
Cross-Pacific paths can't sustain that.

### What this ticket ships

Three additional bounds on the per-topic message cache:

1. **Age cap** — drop entries older than `MAX_CACHE_AGE_SECS` (10 min)
2. **Bytes cap** — drop entries to keep total per-topic bytes ≤ 16 MB
3. **Count cap** — existing 2048 limit retained as a final hard upper bound

Plus per-topic diagnostics so we can see the caps engaging in
production: `msg_count`, `total_bytes`, `oldest_age_secs`,
`evicted_by_age`, `evicted_by_bytes`, `evicted_by_count`.

---

## 1. Investigation phase (do this FIRST, ~30 min)

Confirm or contradict the assumptions below before designing the
implementation. If any assumption is wrong, update this plan in-place
and proceed with the corrected model.

### 1a. Confirm the current cache shape

```bash
cd /Users/davidirvine/Desktop/Devel/projects/saorsa-gossip

# Confirm MAX_CACHE_SIZE and how the LruCache is wired
grep -n "MAX_CACHE_SIZE\|REPLAY_CACHE_MAX_ENTRIES\|message_cache:\|replay_cache:" crates/pubsub/src/lib.rs

# Confirm there's no existing age or bytes bound
grep -n "expires_at\|MAX_AGE\|MAX_BYTES\|total_bytes\|insert_time" crates/pubsub/src/lib.rs
```

Expected: count-only bound today, no time/bytes infrastructure.

### 1b. Confirm topic list and message size profile

```bash
cd /Users/davidirvine/Desktop/Devel/projects/x0x

# Find all topics x0x publishes/subscribes to
grep -rnE 'subscribe\("x0x\.|publish\("x0x\.|TOPIC.*=.*"x0x\.' src/ | grep -v test | head -30

# Look at the most-recent nyc soak proof for actual message sizes
head -100 proofs/launch-readiness-soak-20260512T093455Z-4h-v0_19_41-rollback-98pct-certification/windows/006/runs/baseline/phase-a.log 2>/dev/null || \
  echo "Soak proof dir name will differ on your run — list proofs/ and pick the most recent"
```

Expected topics include at minimum:
- `x0x.discovery.groups` — the big one (11-16 KB per message)
- `x0x.identity.announce` — small, low frequency
- `x0x.presence.beacon` — small, periodic
- `x0x.test.*` — Phase A control topics

### 1c. Audit the x0x application-layer cache

The pubsub-layer cache is what saorsa-gossip stores for IHAVE/IWANT
anti-entropy. **x0x ALSO maintains** application-layer caches of
discovered group cards, contacts, etc. Check those for unbounded growth:

```bash
cd /Users/davidirvine/Desktop/Devel/projects/x0x

# Find the group card store (x0x's own cache, separate from pubsub)
grep -rnE "GroupCardStore|group_card_cache|discovered_group_cards|cache_card\(" src/ | grep -v test | head -10

# Look for any unbounded HashMap/Vec storing gossip-derived state
grep -rnE "HashMap<.*GroupCard|Vec<GroupCard|cards: " src/groups/ src/lib.rs 2>/dev/null | grep -v test | head -10
```

If x0x's group card store is also unbounded, **this ticket should bound
it too** — same age + bytes + count pattern. If it's already bounded,
note the existing bounds in the implementation section.

### 1d. Confirm the diagnostics surface

```bash
cd /Users/davidirvine/Desktop/Devel/projects/x0x

# Find the /diagnostics/gossip handler
grep -nE "diagnostics_gossip|/diagnostics/gossip|gossip_diagnostics" src/bin/x0xd.rs | head -5

# Find the type that backs it (GossipDiagnostics or similar)
grep -rn "pub struct.*GossipDiagnostics\|gossip_diagnostics_snapshot" src/ | head -5
```

The plan assumes there's an existing `/diagnostics/gossip` endpoint we
can extend with per-topic cache stats. If it doesn't exist, you'll need
to either (a) add it as part of this ticket, or (b) extend the existing
`/diagnostics/gossip` route to include the new fields. Adjust the
"Files to modify" section accordingly.

### 1e. Note the running soak (if applicable)

If a soak is currently running on the fleet, **wait for it to complete
or explicitly stop it before deploying changes**. The proof artefact is
more valuable than rushing the next iteration. The 4h soak takes ~4h
wall clock; check with:

```bash
ps aux | grep launch_soak | grep -v grep
```

---

## 2. Design

### 2a. Constants

In `saorsa-gossip/crates/pubsub/src/lib.rs`, alongside the existing
`MAX_CACHE_SIZE`:

```rust
/// Maximum age of a cached pubsub message before forced eviction.
/// Calibrated for cross-region p2p mesh where anti-entropy must not
/// reach back further than this. 10 min covers retransmit + IHAVE
/// dedupe windows with margin.
const MAX_CACHE_AGE_SECS: u64 = 600;

/// Maximum total bytes of cached messages per topic. Cross-Pacific
/// paths (~280 ms RTT) can sustain ~10 MB/s anti-entropy reconcile;
/// 16 MB per topic gives a worst-case 1.6 s anti-entropy cycle.
const MAX_CACHE_BYTES_PER_TOPIC: usize = 16 * 1024 * 1024;
```

Document the calibration source in the doc comments. Both numbers
should be configurable via the existing pubsub config struct (if any)
or new fields on `GossipRuntimeConfig` — the operator must be able to
tune these without recompiling.

### 2b. Cache data structure

Replace the bare `LruCache<MessageIdType, CachedMessage>` with a
`BoundedMessageCache` struct that wraps it and adds the new bounds:

```rust
/// Per-topic message cache with count + bytes + age bounds.
/// Eviction priority: age → bytes → count (newest-eligible last).
pub(crate) struct BoundedMessageCache {
    lru: LruCache<MessageIdType, CachedEntry>,
    total_bytes: usize,
    max_count: NonZeroUsize,
    max_bytes: usize,
    max_age: Duration,
    /// Eviction counters for diagnostics.
    evicted_by_age: u64,
    evicted_by_bytes: u64,
    evicted_by_count: u64,
}

pub(crate) struct CachedEntry {
    pub message: CachedMessage,
    pub bytes: usize,           // CachedMessage.payload.len() + header overhead
    pub inserted_at: Instant,
}

impl BoundedMessageCache {
    pub fn new(max_count: NonZeroUsize, max_bytes: usize, max_age: Duration) -> Self { ... }

    pub fn insert(&mut self, id: MessageIdType, msg: CachedMessage) -> Option<CachedMessage> {
        let bytes = estimate_message_bytes(&msg);
        self.prune_expired(Instant::now());
        self.ensure_bytes_capacity(bytes);
        self.ensure_count_capacity();
        let entry = CachedEntry { message: msg, bytes, inserted_at: Instant::now() };
        self.total_bytes = self.total_bytes.saturating_add(bytes);
        let evicted = self.lru.push(id, entry);
        if let Some((_, old)) = evicted {
            self.total_bytes = self.total_bytes.saturating_sub(old.bytes);
        }
        evicted.map(|(_, e)| e.message)
    }

    pub fn get(&mut self, id: &MessageIdType) -> Option<&CachedMessage> {
        self.prune_expired(Instant::now());
        self.lru.get(id).map(|e| &e.message)
    }

    fn prune_expired(&mut self, now: Instant) {
        while let Some((_, entry)) = self.lru.peek_lru() {
            if now.saturating_duration_since(entry.inserted_at) >= self.max_age {
                let (_, e) = self.lru.pop_lru().expect("peek_lru just succeeded");
                self.total_bytes = self.total_bytes.saturating_sub(e.bytes);
                self.evicted_by_age += 1;
            } else {
                break;
            }
        }
    }

    fn ensure_bytes_capacity(&mut self, incoming_bytes: usize) {
        while self.total_bytes + incoming_bytes > self.max_bytes {
            let Some((_, e)) = self.lru.pop_lru() else { break };
            self.total_bytes = self.total_bytes.saturating_sub(e.bytes);
            self.evicted_by_bytes += 1;
        }
    }

    fn ensure_count_capacity(&mut self) {
        while self.lru.len() >= self.max_count.get() {
            let Some((_, e)) = self.lru.pop_lru() else { break };
            self.total_bytes = self.total_bytes.saturating_sub(e.bytes);
            self.evicted_by_count += 1;
        }
    }

    pub fn stats(&self) -> CacheStats {
        let oldest_age_secs = self.lru.peek_lru()
            .map(|(_, e)| e.inserted_at.elapsed().as_secs())
            .unwrap_or(0);
        CacheStats {
            msg_count: self.lru.len(),
            total_bytes: self.total_bytes,
            oldest_age_secs,
            evicted_by_age: self.evicted_by_age,
            evicted_by_bytes: self.evicted_by_bytes,
            evicted_by_count: self.evicted_by_count,
        }
    }
}
```

**Key design decisions** (justify in doc comments):

- **Eviction order: age → bytes → count.** Age first because we want
  freshness guarantees independent of load; bytes second because that's
  the resource we're protecting; count last as a hard upper bound.
- **`prune_expired` runs on every `insert` and `get`.** Cheap because
  it's `O(expired count)` from the LRU tail. Could be amortised but
  not necessary at our scale.
- **`saturating_sub` on `total_bytes`.** Defensive — if the byte
  estimate ever goes out of sync with the LRU contents, we degrade to
  "slightly off accounting" not "panic".
- **Counters are `u64` and monotonic.** Diagnostics consumers track
  *deltas* over windows, never assume reset.

### 2c. Per-topic byte estimation

`estimate_message_bytes(&CachedMessage) -> usize` should include:

- Payload size (`message.payload.len()`)
- Header overhead estimate (constant `MESSAGE_HEADER_OVERHEAD_BYTES = 256`)
- Signature size (constant, ~3300 bytes for ML-DSA-65)

Keep it simple — over-estimate slightly is fine, under-estimate is bad.

### 2d. Topic registry

If `pubsub/src/lib.rs` already has a per-topic state map (likely keyed
by `TopicId`), thread the `BoundedMessageCache` through there. If
caches are global across topics, **change to per-topic** — the bounds
should apply per-topic, not across all topics. The soak data shows the
problem is concentrated on `x0x.discovery.groups`; per-topic bounding
keeps small/sparse topics from being evicted by the noisy big one.

### 2e. Diagnostics surface

Extend the existing per-topic stats with the new cache fields:

```rust
// In saorsa-gossip's diagnostics module:
pub struct TopicDiagnostics {
    pub topic_id: String,
    pub subscriber_count: usize,
    // existing fields...
    pub cache: CacheStats,         // NEW
}

pub struct CacheStats {
    pub msg_count: usize,
    pub total_bytes: usize,
    pub oldest_age_secs: u64,
    pub evicted_by_age: u64,
    pub evicted_by_bytes: u64,
    pub evicted_by_count: u64,
}
```

In `x0x/src/bin/x0xd.rs` (`/diagnostics/gossip` handler), surface the
new fields. They'll automatically flow through to the existing
`launch_readiness.py` collection path which serialises everything in
`/diagnostics/gossip`.

---

## 3. Implementation tasks

Work in this order. **Each step ends with green tests before moving
on.** Do not bundle commits.

### Step 1: saorsa-gossip — BoundedMessageCache core

**Repo:** `saorsa-gossip`
**Files:** `crates/pubsub/src/lib.rs` (or split into
`crates/pubsub/src/bounded_cache.rs` if it gets > 200 lines)

- [ ] Add `MAX_CACHE_AGE_SECS`, `MAX_CACHE_BYTES_PER_TOPIC` constants
      with doc comments referencing this plan
- [ ] Define `BoundedMessageCache`, `CachedEntry`, `CacheStats` per §2b
- [ ] Define `estimate_message_bytes` per §2c
- [ ] Replace `LruCache<MessageIdType, CachedMessage>` field with
      `BoundedMessageCache`
- [ ] Update all `message_cache.put(...)`, `message_cache.get(...)`,
      `message_cache.len()` call sites to use new API
- [ ] Add unit tests in same file:
  - `bounded_cache_evicts_by_age`
  - `bounded_cache_evicts_by_bytes_under_pressure`
  - `bounded_cache_evicts_by_count_hard_cap`
  - `bounded_cache_age_takes_precedence_over_bytes`
  - `bounded_cache_eviction_counters_track_correctly`
  - `bounded_cache_get_prunes_expired`

**Acceptance:**
```bash
cd /Users/davidirvine/Desktop/Devel/projects/saorsa-gossip
cargo fmt --all -- --check
cargo clippy --all-features --all-targets -- -D warnings
cargo nextest run --all-features -E 'test(bounded_cache_)'  # all new tests pass
cargo nextest run --all-features --workspace                # no regressions
```

**Commit:** `feat(pubsub): X0X-0068 — BoundedMessageCache with age + bytes + count caps`

### Step 2: saorsa-gossip — per-topic stats wiring

**Repo:** `saorsa-gossip`
**Files:** `crates/pubsub/src/diagnostics.rs` (or wherever
`TopicDiagnostics` lives)

- [ ] Add `CacheStats` to `TopicDiagnostics`
- [ ] Wire `BoundedMessageCache::stats()` into the snapshot path
- [ ] Update any existing serialization (JSON / serde) to include the
      new fields
- [ ] Update existing diagnostics tests to assert the new fields are
      populated

**Acceptance:** workspace tests still pass; the JSON snapshot for a
topic includes `cache.msg_count`, etc.

**Commit:** `feat(pubsub): X0X-0068 — expose per-topic cache stats in TopicDiagnostics`

### Step 3: saorsa-gossip — release + version bump

Follow the standard saorsa-gossip release process:

- [ ] `Cargo.toml` version bump (workspace + all 11 crates)
- [ ] `CHANGELOG.md` entry under `[Unreleased] → [<new version>]`
- [ ] `git tag v<new version>`, push tag, wait for crates.io publish
- [ ] Verify on crates.io before proceeding to x0x

### Step 4: x0x — pick up new saorsa-gossip + diagnostics surface

**Repo:** `x0x`
**Files:** `Cargo.toml`, `src/bin/x0xd.rs` (the `/diagnostics/gossip`
handler), `tests/test_launch_readiness.py` (assertions)

- [ ] Bump all `saorsa-gossip-* = "<new version>"` in `Cargo.toml`
- [ ] `cargo update -p saorsa-gossip-pubsub` (etc.)
- [ ] Modify `/diagnostics/gossip` handler to include per-topic cache
      stats in the JSON response
- [ ] Update `tests/launch_readiness.py` if it does any cache-stats
      parsing; otherwise the stats just flow through to the proof
      artefacts

**Acceptance:**
```bash
cd /Users/davidirvine/Desktop/Devel/projects/x0x
cargo fmt --all -- --check
cargo clippy --all-features --all-targets -- -D warnings
cargo nextest run --all-features --workspace        # 1164+/1164 (or more) pass
python3 -m unittest tests.test_launch_readiness     # 22/22 pass
python3 -m unittest tests.test_launch_soak          # 15/15 pass
```

**Commit:** `feat(x0x): X0X-0068 — bump saorsa-gossip to <new ver>; expose per-topic cache stats on /diagnostics/gossip`

### Step 5: audit x0x application-layer cache (if needed)

From investigation §1c, if x0x's `GroupCardStore` (or equivalent) is
also unbounded, bound it with the same pattern:

- [ ] Apply LRU + age + bytes bounds to the x0x-side cache
- [ ] Surface stats on a relevant diagnostics endpoint (likely
      `/diagnostics/groups` or `/diagnostics/discovery`)
- [ ] Tests as for §1

If the x0x-side cache is already bounded or doesn't exist as a separate
cache from the pubsub cache, **skip this step** and note in the ticket
addendum that no x0x-side changes were needed.

---

## 4. Validation

### 4a. Local validation

```bash
# saorsa-gossip
cd /Users/davidirvine/Desktop/Devel/projects/saorsa-gossip
cargo fmt --all -- --check
cargo clippy --all-features --all-targets -- -D warnings
cargo nextest run --all-features --workspace

# x0x
cd /Users/davidirvine/Desktop/Devel/projects/x0x
cargo fmt --all -- --check
cargo clippy --all-features --all-targets -- -D warnings
cargo nextest run --all-features --workspace
python3 -m unittest tests.test_launch_readiness
python3 -m unittest tests.test_launch_soak
```

All clean before deploying.

### 4b. Deploy cycle

```bash
cd /Users/davidirvine/Desktop/Devel/projects/x0x

# Cross-compile for Linux
cargo zigbuild --release --target x86_64-unknown-linux-gnu --bin x0xd

# Deploy to all 6 nodes (this restarts x0xd + the test runner on each)
bash tests/e2e_deploy.sh

# Expect: ALL 24 CHECKS PASSED
```

### 4c. Idle drain (mandatory — see X0X-0067)

The fleet needs ≥ 1 hour idle after the deploy before a soak will
produce clean evidence. **Do not skip this step** — it's the root cause
of half of yesterday's wasted soak cycles.

```bash
echo "drain start: $(date -u +%H:%M:%SZ)"
sleep 3600
echo "drain end: $(date -u +%H:%M:%SZ)"
```

### 4d. 1h confirmatory soak

```bash
SOAK_DIR="proofs/launch-readiness-soak-$(date -u +%Y%m%dT%H%M%SZ)-1h-v$VERSION-x0x-0068-1h"
mkdir -p "$SOAK_DIR"
python3 tests/launch_soak.py \
    --duration-hours 1 \
    --interval-mins 15 \
    --anchor nyc \
    --gate broad-launch \
    --soak-dir "$SOAK_DIR" 2>&1 | tee "${SOAK_DIR}.runner.log"

# Re-evaluate under the aggregate Phase A SLO
python3 -c "
import csv, importlib.util
from pathlib import Path
spec = importlib.util.spec_from_file_location('launch_soak', 'tests/launch_soak.py')
mod = importlib.util.module_from_spec(spec); spec.loader.exec_module(mod)
soak = Path('$SOAK_DIR')
with (soak / 'timeline.csv').open(newline='') as f:
    rows = list(csv.DictReader(f))
for idx, r in enumerate(rows, 1):
    annotated = mod.discover_windows_summary(soak / 'windows' / f'{idx:03d}')
    r['violation_messages'] = annotated.get('violation_messages', '')
passed = mod.write_summary(soak, 'broad-launch', rows)
print('PASS' if passed else 'FAIL')
print((soak / 'summary.md').read_text()[:3500])
"
```

**Acceptance for 1h soak:**
- `overall_pass == True` (i.e. soak summary writes "Overall verdict: GO")
- Aggregate Phase A sent ≥ 98% AND received ≥ 98%
- `drop_full == 0` across all windows
- Per-topic `evicted_by_age` or `evicted_by_bytes` > 0 on at least one
  window (proves the new caps are engaging)
- `max_pp_to` in the final window ≤ 2× `max_pp_to` in window 1

If 1h fails: **stop, don't escalate to 4h**. Investigate the failure
pattern; the cache fix should make 1h *easier* not harder. Common
failure modes:
- Eviction counters all zero → caps not engaging (check the wiring)
- `drop_full > 0` → some other backpressure mechanism is now the binding
  constraint (file X0X-006X follow-up)
- `max_pp_to` still climbs → the cache wasn't actually the problem; need
  to instrument anti-entropy further

### 4e. 4h certification soak (only after 1h passes)

```bash
SOAK_DIR="proofs/launch-readiness-soak-$(date -u +%Y%m%dT%H%M%SZ)-4h-v$VERSION-x0x-0068-cert"
mkdir -p "$SOAK_DIR"
python3 tests/launch_soak.py \
    --duration-hours 4 \
    --interval-mins 15 \
    --anchor nyc \
    --gate broad-launch \
    --soak-dir "$SOAK_DIR" 2>&1 | tee "${SOAK_DIR}.runner.log"
```

**Acceptance for 4h cert soak (the actual portfolio gate):**
- `overall_pass == True`
- Aggregate Phase A sent ≥ 98% AND received ≥ 98%
- `drop_full == 0` across all 16 windows
- Window 16's `max_pp_to` ≤ 2× window 1's `max_pp_to` (the **plateau**
  acceptance — this is the proof that the feedback loop is broken)
- Per-topic `evicted_by_age` and `evicted_by_bytes` counters both > 0
  in the final summary (proves caps actively engaging)

---

## 5. Telemetry to inspect after each soak

```bash
SOAK=proofs/launch-readiness-soak-<your-run-id>

# Eviction counters across windows
for w in $SOAK/windows/0[01-9][1-6]; do
    echo "=== window $(basename $w) ==="
    grep -E "evicted_by|cache.*msg_count|cache.*total_bytes|oldest_age" "$w/diagnostics/baseline/"*.json 2>/dev/null | head -5
done

# Cache size growth pattern (should plateau, not climb)
grep -E "total_bytes" $SOAK/windows/*/diagnostics/baseline/*.json | head -20

# Cross-region peer cooling
grep -E "Peer cooled" $SOAK/windows/*/runs/baseline/*.log | head -20
```

The shape you want to see: total_bytes growth flattens around windows
2-3 (caches reach capacity), then evicted_by counters tick up steadily,
and per-peer-timeout stays low and stable.

---

## 6. Rollback plan

If 1h soak fails AND investigation shows the fix made things worse
(not "same as before"), rollback is:

```bash
# Revert the saorsa-gossip changes
cd /Users/davidirvine/Desktop/Devel/projects/saorsa-gossip
git revert <bounded-cache-commit>
git tag v<prev-version>; git push --tags  # re-publish prior version

# Bump x0x's saorsa-gossip dep back
cd /Users/davidirvine/Desktop/Devel/projects/x0x
git revert <bump-commit>

# Redeploy
cargo zigbuild --release --target x86_64-unknown-linux-gnu --bin x0xd
bash tests/e2e_deploy.sh

# Document the failure mode in X0X-0068 ticket; reopen as "rejected, needs different design"
```

The portfolio plan ([sota-borrow-phase-2-fleet-survival.md](sota-borrow-phase-2-fleet-survival.md))
sequencing assumes X0X-0068 passes first; on rollback, escalate to the
coordinator.

---

## 7. Ticket close checklist

When everything passes:

- [ ] All commits pushed to `main` on both repos
- [ ] saorsa-gossip released to crates.io
- [ ] 1h soak proof committed (with `summary.md` showing GO)
- [ ] 4h soak proof committed (with `summary.md` showing GO + plateau)
- [ ] `issues/issues.jsonl` X0X-0068 ticket: `state` → `done`,
      addendum referencing both proofs and the cache stats showing
      caps engaged
- [ ] `docs/launch-gates/broad-launch.md` updated to note the bounded
      cache as a steady-state expectation
- [ ] Unblock Track A: X0X-0073 (adaptive cooling) can now start

---

## Appendix: relevant code locations (snapshot 2026-05-12)

```
saorsa-gossip/crates/pubsub/src/lib.rs:42      const MAX_CACHE_SIZE: usize = 2_048
saorsa-gossip/crates/pubsub/src/lib.rs:53      const REPLAY_CACHE_MAX_ENTRIES: usize = 10_000
saorsa-gossip/crates/pubsub/src/lib.rs:1045    fn message_cache_capacity()
saorsa-gossip/crates/pubsub/src/lib.rs:1050    fn replay_cache_capacity()
saorsa-gossip/crates/pubsub/src/lib.rs:1405    message_cache: LruCache<MessageIdType, CachedMessage>
saorsa-gossip/crates/pubsub/src/lib.rs:1418    replay_cache:  LruCache<[u8; 32], Instant>

x0x/src/bin/x0xd.rs                            /diagnostics/gossip handler
x0x/tests/launch_soak.py:39                    SOAK_MAX_DISPATCHER_TIMED_OUT_DELTA_PER_12H = 5
x0x/tests/launch_soak.py:53                    SOAK_MIN_AGGREGATE_PHASE_A_RATIO = 0.98
x0x/tests/launch_readiness.py:66               GATES dict (broad-launch SLO)
```

Line numbers may have shifted — re-grep on your branch before editing.
