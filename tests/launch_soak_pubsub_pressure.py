#!/usr/bin/env python3
"""PubSub-pressure-only soak for X0X-0076.

This is Variant B of the split-soak methodology. It runs repeated
`launch_readiness.py --scenarios fanout_burst` windows and intentionally
does not run the direct-DM Phase A matrix, so suppression growth can be
attributed to overlay pressure rather than transport/DM failures.
"""
from __future__ import annotations

import argparse
import csv
import logging
import re
import shlex
import signal
import subprocess
import sys
import time
from pathlib import Path
from typing import Dict, List, Optional

LOG = logging.getLogger("launch_soak_pubsub_pressure")


def _max_int(rows: List[Dict[str, str]], field: str) -> int:
    values = []
    for row in rows:
        try:
            values.append(max(0, int(row.get(field, "0") or "0")))
        except ValueError:
            pass
    return max(values, default=0)


def _max_float(rows: List[Dict[str, str]], field: str) -> str:
    values = []
    for row in rows:
        raw = row.get(field, "0") or "0"
        try:
            values.append(float("inf") if raw == "inf" else float(raw))
        except ValueError:
            pass
    if not values:
        return "0.000000"
    value = max(values)
    return "inf" if value == float("inf") else f"{value:.6f}"


def summarize_window(window_dir: Path) -> Dict[str, str]:
    summary_path = window_dir / "summary.md"
    csv_path = window_dir / "summary.csv"
    row = {
        "verdict": "MISSING",
        "max_disp_to_delta": "0",
        "max_drop_full_delta": "0",
        "max_pp_to_delta": "0",
        "max_suppressed": "0",
        "max_suppressed_ratio": "0.000000",
        "max_workers": "0",
        "violations": "0",
    }

    if summary_path.exists():
        text = summary_path.read_text(encoding="utf-8", errors="replace")
        match = re.search(r"Overall verdict:\s*\*\*(GO|NO-GO)\*\*", text)
        if match:
            row["verdict"] = match.group(1)

    if not csv_path.exists():
        return row

    with csv_path.open(newline="") as f:
        rows = [r for r in csv.DictReader(f) if r.get("scenario") == "fanout_burst"]
    if not rows:
        return row

    row.update({
        "max_disp_to_delta": str(_max_int(rows, "dispatcher_timed_out_delta")),
        "max_drop_full_delta": str(_max_int(rows, "recv_pump_dropped_full_delta")),
        "max_pp_to_delta": str(_max_int(rows, "per_peer_timeout_delta")),
        "max_suppressed": str(_max_int(rows, "suppressed_peers_post")),
        "max_suppressed_ratio": _max_float(rows, "suppressed_peers_to_known_ratio"),
        "max_workers": str(_max_int(rows, "pubsub_workers_post")),
        "violations": str(_max_int(rows, "violations_count")),
    })
    return row


def run_window(repo_root: Path, window_dir: Path, args: argparse.Namespace) -> int:
    cmd = [
        sys.executable,
        str(repo_root / "tests" / "launch_readiness.py"),
        "--gate", args.gate,
        "--scenarios", "fanout_burst",
        "--anchor", args.anchor,
        "--proof-dir", str(window_dir),
        "--burst-messages", str(args.burst_messages),
        "--burst-payload-bytes", str(args.burst_payload_bytes),
        "--burst-delay-ms", str(args.burst_delay_ms),
    ]
    LOG.info("window run: %s", " ".join(shlex.quote(c) for c in cmd))
    proc = subprocess.run(cmd, cwd=repo_root, capture_output=True, timeout=args.window_timeout_secs)
    (window_dir / "stdout.log").write_bytes(proc.stdout)
    (window_dir / "stderr.log").write_bytes(proc.stderr)
    return proc.returncode


def write_summary(soak_dir: Path, rows: List[Dict[str, str]]) -> bool:
    pass_count = sum(1 for r in rows if r["verdict"] == "GO")
    fail_count = sum(1 for r in rows if r["verdict"] == "NO-GO")
    missing_count = sum(1 for r in rows if r["verdict"] == "MISSING")
    max_suppressed = max((int(r["max_suppressed"]) for r in rows), default=0)
    max_pp_to = max((int(r["max_pp_to_delta"]) for r in rows), default=0)
    max_drop_full = max((int(r["max_drop_full_delta"]) for r in rows), default=0)
    passed = missing_count == 0 and fail_count == 0

    lines = [
        "# PubSub Pressure Soak Summary",
        "",
        f"- verdict: **{'PASS' if passed else 'FAIL'}**",
        f"- windows: {len(rows)} (pass={pass_count}, fail={fail_count}, missing={missing_count})",
        f"- max_suppressed: {max_suppressed}",
        f"- max_pp_to_delta: {max_pp_to}",
        f"- max_drop_full_delta: {max_drop_full}",
        "",
        "| window | verdict | max_suppressed | max_suppressed_ratio | max_pp_to_delta | max_drop_full_delta | violations |",
        "|---:|---|---:|---:|---:|---:|---:|",
    ]
    for idx, row in enumerate(rows, 1):
        lines.append(
            f"| {idx} | {row['verdict']} | {row['max_suppressed']} | "
            f"{row['max_suppressed_ratio']} | {row['max_pp_to_delta']} | "
            f"{row['max_drop_full_delta']} | {row['violations']} |"
        )
    (soak_dir / "summary.md").write_text("\n".join(lines), encoding="utf-8")
    return passed


def main(argv: Optional[List[str]] = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--duration-hours", type=float, default=4.0)
    parser.add_argument("--interval-mins", type=float, default=15.0)
    parser.add_argument("--anchor", default="nyc")
    parser.add_argument("--gate", default="broad-launch")
    parser.add_argument("--soak-dir", default=None)
    parser.add_argument("--burst-messages", type=int, default=5000)
    parser.add_argument("--burst-payload-bytes", type=int, default=4096)
    parser.add_argument("--burst-delay-ms", type=int, default=1)
    parser.add_argument("--window-timeout-secs", type=int, default=1200)
    args = parser.parse_args(argv)

    logging.basicConfig(level=logging.INFO, format="%(asctime)s %(levelname)s %(message)s")
    repo_root = Path(__file__).resolve().parents[1]
    ts = time.strftime("%Y%m%dT%H%M%SZ", time.gmtime())
    soak_dir = Path(args.soak_dir) if args.soak_dir else (
        repo_root / "proofs" / f"launch-soak-pubsub-pressure-{ts}"
    )
    soak_dir.mkdir(parents=True, exist_ok=True)
    (soak_dir / "windows").mkdir(exist_ok=True)

    interrupted = {"flag": False}

    def _stop(signum: int, _frame) -> None:
        interrupted["flag"] = True
        LOG.warning("signal %d caught; finishing current window", signum)

    signal.signal(signal.SIGINT, _stop)
    signal.signal(signal.SIGTERM, _stop)

    target_windows = max(1, int((args.duration_hours * 3600) / (args.interval_mins * 60)))
    rows: List[Dict[str, str]] = []
    timeline_path = soak_dir / "timeline.csv"
    with timeline_path.open("w", newline="") as f:
        writer = csv.writer(f)
        writer.writerow([
            "window", "start_unix", "verdict", "max_disp_to_delta",
            "max_drop_full_delta", "max_pp_to_delta", "max_suppressed",
            "max_suppressed_ratio", "max_workers", "violations", "window_rc",
        ])

    for idx in range(1, target_windows + 1):
        if interrupted["flag"]:
            break
        started = time.time()
        window_dir = soak_dir / "windows" / f"{idx:03d}"
        window_dir.mkdir(parents=True, exist_ok=True)
        rc = run_window(repo_root, window_dir, args)
        row = summarize_window(window_dir)
        row["start_unix"] = str(int(started))
        row["window_rc"] = str(rc)
        rows.append(row)

        with timeline_path.open("a", newline="") as f:
            writer = csv.writer(f)
            writer.writerow([
                idx, row["start_unix"], row["verdict"], row["max_disp_to_delta"],
                row["max_drop_full_delta"], row["max_pp_to_delta"],
                row["max_suppressed"], row["max_suppressed_ratio"],
                row["max_workers"], row["violations"], row["window_rc"],
            ])

        sleep_for = (args.interval_mins * 60) - (time.time() - started)
        if idx < target_windows and sleep_for > 0 and not interrupted["flag"]:
            time.sleep(sleep_for)

    return 0 if write_summary(soak_dir, rows) else 1


if __name__ == "__main__":
    sys.exit(main())
