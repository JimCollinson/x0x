# X0X-0024 investigation: dispatcher-timeout cadence and window 20 spike

Date: 2026-05-05

## Classification

The reported window 20 Singapore spike is a harness/snapshot artifact.
The broader overnight soak did have real background dispatcher-timeout
movement, but it was not a 3h cadence and it was not a Singapore-local
dispatcher burst during the measured scenario.

Do not tune the broad-launch soak cap to this VPS mesh. The portable
lesson is that long-soak evidence must be continuous and normalized by
actual work volume/topology, while node behavior must remain adaptive
through cooling/scoring/budgeting rather than operator-selected VPS
constants.

## Key findings

- Strict delivery/drop bars stayed healthy: Phase A was 600/600 across
  20 windows and `recv_pump.dropped_full` stayed 0.
- Window 20 had no `singapore-pre.json`. The pre-snapshot fetch timed
  out before Phase A, then `launch_readiness.py` diffed `{}` against
  `singapore-post.json`. That made Singapore lifetime counters look like
  window deltas: `dispatcher_timed_out +4` and `per_peer_timeout +15282`.
- Bridging from window 19 post to window 20 post gives the real
  Singapore interval delta: `dispatcher_timed_out +0`,
  `per_peer_timeout +184`.
- The real continuous window 20 dispatcher movement was `+4` across
  other nodes between post snapshots: nyc `+1`, nuremberg `+3`,
  Singapore `+0`.
- The apparent 3h cadence is sampling aliasing from the scenario-only
  summary. Continuous post-to-post accounting shows dispatcher timeout
  movement in most late windows, especially Helsinki and Nuremberg, not
  isolated events only in windows 6/12/17/20.

## Continuous counter totals

Computed from each node's previous successful post snapshot to the next
post snapshot:

| Metric | Continuous total |
|---|---:|
| `dispatcher.pubsub.timed_out` | 75 |
| `dispatcher.pubsub.completed` | 29,781,203 |
| `dispatcher_timeout / dispatcher_completed` | 0.00000252 |
| `republish_per_peer_timeout` | 131,148 |
| `per_peer_timeout / dispatcher_completed` | 0.004404 |
| `recv_pump.dropped_full` | 0 |
| unaccounted telemetry gaps | 0 |

Window 20 has one accounted telemetry gap, `singapore:pre`: the pre
snapshot was missing, but the previous post snapshot was available, so
the interval delta remains measurable.

## Windows of interest

| Window | Scenario disp_to | Continuous disp_to | Scenario max pp_to | Continuous max pp_to | Notes |
|---:|---:|---:|---:|---:|---|
| 6 | 1 | 4 | 170 | 2922 | Scenario caught Helsinki +1; continuous interval also includes earlier movement. |
| 12 | 1 | 4 | 150 | 2871 | Scenario caught Nuremberg +1; continuous interval includes Helsinki +1 and Nuremberg +3. |
| 17 | 1 | 5 | 111 | 2797 | Scenario caught Helsinki +1; continuous interval includes nyc +1, Helsinki +1, Nuremberg +3. |
| 20 | 4 | 4 | 15282 | 2762 | Scenario spike is synthetic from missing Singapore pre; continuous movement is nyc +1 and Nuremberg +3. |

## Journal evidence

Targeted `journalctl -u x0xd` slices were pulled for the requested
windows with bounded remote timeouts.

The slices did not show explicit dispatcher-timeout log lines matching
the counter increments, which means the diagnostics counters are the
authoritative signal for this class. Singapore's window 20 journal did
show a burst of `IWANT for unknown message` warnings around
`2026-05-05T07:08:44Z`. That is consistent with background anti-entropy
or cache-window pressure, but it does not explain the reported
`+4/+15282` Singapore delta because the diagnostics bridge proves those
Singapore values were lifetime-counter artifacts.

Match counts for `timeout|timed_out|dispatcher|cool|suppress|warn|error|IWANT for unknown`
in the requested journal slices:

| Window | helsinki | nuremberg | nyc | sfo | singapore | sydney |
|---|---:|---:|---:|---:|---:|---:|
| w06 00:10-00:20Z | 0 | 0 | 0 | 0 | 0 | 0 |
| w12 03:10-03:20Z | 0 | 0 | 1044 | 2828 | 0 | 0 |
| w17 05:40-05:50Z | 0 | 0 | 1409 | 2939 | 1136 | 3456 |
| w20 07:00-07:30Z | 0 | 0 | 1325 | 2290 | 1537 | 5091 |

Those WARN bursts are an investigation signal for gossip cache/IWANT
behavior, but they are not delivery failures: every Phase A window still
completed 30/30 and drops stayed 0.

## Decision

X0X-0024 should close as a harness-accounting fix plus a follow-on
adaptive-gate/product-runtime item:

1. `tests/launch_soak.py` now annotates rows with continuous
   post-to-post counter deltas and records snapshot gaps.
2. Missing pre snapshots no longer synthesize lifetime counter deltas.
3. Long-soak policy should not be a fixed dispatcher-timeout count for a
   six-node VPS mesh. It should use normalized rates and adaptive
   baselines, with strict failure still reserved for delivery misses,
   `recv_pump.dropped_full`, unaccounted telemetry gaps, sustained
   backlog, or timeout bursts coupled to degraded delivery.

Recommended follow-on: file and implement an adaptive long-soak gate
that learns a warmed baseline, evaluates normalized timeout rates per
completed dispatch and per node-hour, requires N consecutive anomalous
windows before failing on dispatcher-only noise, and keeps delivery/drop
bars strict.
