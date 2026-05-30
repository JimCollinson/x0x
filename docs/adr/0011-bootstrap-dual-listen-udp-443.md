# 0011 — Bootstrap nodes dual-listen on UDP/443; clients dial 443 first and never bind privileged ports

- Status: Accepted
- Date: 2026-05-30

## Context

x0x's transport is QUIC over UDP (via `ant-quic`), with bootstrap, relay, and
gossip all on **UDP 5483**. Users behind full-tunnel VPNs (notably Cloudflare
WARP / "1.1.1.1: Faster Internet"), corporate/hotel/captive networks, mobile
carriers, and CGNAT report that x0x cannot connect. Investigation
(see [[ackv2-empty-response-no-retry-2026-05-29]] for the related transport
work, and the WARP support thread) shows two mechanisms:

1. **Port filtering / throttling.** These networks carry UDP/443 (mainstream
   HTTP/3) cleanly but throttle or drop arbitrary high UDP ports like 5483.
2. **MTU.** WireGuard-style tunnels shrink the path MTU; QUIC's mandatory
   1200-byte Initial (`ant-quic` `MIN_INITIAL_SIZE = 1200`) can be dropped,
   so the handshake never completes. The 443 path is usually MTU-tuned because
   it is the VPN's optimized hot path.

A naive reading is "move x0x to UDP/443." The decisive nuance: **for traversal,
what matters is the *destination* port a client dials, not the client's own
*listen* port.** Egress filtering is destination-based; a client behind a
hostile network makes *outbound* connections (ephemeral high source port → no
privilege needed) and, behind WARP/symmetric NAT, cannot receive inbound at all
(it relays — handled by the existing X0X-0070 peer relay + ant-quic MASQUE).

Binding a *listener* on UDP/443 requires privilege (<1024 ⇒ root /
`CAP_NET_BIND_SERVICE` on Linux, root on macOS; Windows excepted). x0x is a
user-run daemon (`~/.x0x`); requiring elevation for every client is a security
and UX regression — and buys nothing, because dialing a low destination port is
unprivileged and inbound is relayed anyway.

## Decision

1. **Bootstrap and relay nodes dual-listen on UDP/443 *and* UDP/5483.** They run
   as root on the VPS fleet, so binding 443 is free. Implemented via ant-quic's
   existing `NatTraversalConfig.additional_bind_addrs` (one node identity, an
   additional bound socket on 443 alongside 5483 — analogous to the existing
   dual-stack v4/v6 binding). No new node identity.
2. **Clients never bind a privileged port.** The client listener stays on the
   high port (5483 or ephemeral). `additional_bind_addrs` defaults to empty, so
   client behaviour is unchanged and never needs root.
3. **The bootstrap seed list carries both `IP:443` and `IP:5483`** for each
   node, and the client connect path tries 443 first, falling back to 5483.
   This is what traverses WARP/firewalls/CGNAT (outbound dest = 443).
4. **`x0xd --doctor` / `/diagnostics/connectivity` detect a full-tunnel-VPN /
   constrained-MTU path** (external_addr in a known VPN egress range, low
   `current_mtu` / lost PLPMTUD probes, or `can_receive_direct=false` with
   handshake timeouts) and emit actionable guidance ("full-tunnel VPN detected —
   use split-tunnel / exclude x0x / DNS-only mode"). Turns a silent failure into
   self-service.

## Consequences

- **Pro:** WARP and the broader UDP-hostile-network class (corporate/hotel/
  CGNAT/mobile) can reach the mesh by dialing bootstrap/relay on 443. x0x-on-443
  blends with HTTP/3 at the port level (censorship-resistance bonus). No client
  privilege change.
- **Con / bounded:** MTU is still a hard floor — a path that cannot carry a
  1200-byte datagram cannot run QUIC regardless of port; 443 mitigates
  throttling/DPI and rides a better-tuned path but is not a universal fix. The
  only complete answer for sub-1200-MTU paths is a future TCP/HTTP fallback
  transport (out of scope here).
- **Migration:** bootstrap nodes dual-listen (443 + 5483) so old (5483-only)
  and new clients both connect; seed list keeps 5483 entries. Heterogeneous
  meshes are fine — identity is key-based and actual ports propagate via
  announcements; only the seed list has a fixed assumption, and it carries both.
- **Ops:** open UDP/443 on the bootstrap fleet; ensure nothing else holds
  UDP/443 there (TCP/443 web is independent of UDP/443). Bootstrap-node config
  sets `additional_bind_addrs = ["0.0.0.0:443"]`.

## Supersedes / relates to

- Relates to [[0001-bootstrap-peers-are-seed-hints-only]] (seed list is hints;
  this adds a second port per hint).
- Builds on the existing X0X-0070 application-level peer relay and ant-quic
  MASQUE relay for the no-inbound-reachability case.
