#!/usr/bin/env python3
"""Fixed-roster direct-DM soak for X0X-0076.

This is Variant A of the split-soak methodology. It runs the existing
`launch_soak.py` loop while setting `X0X_NO_PUBSUB_AFTER_DISCOVER=1`, so
the Phase A harness uses PubSub only for initial runner discovery and then
relies on direct-DM control/results for the rest of each window.
"""
from __future__ import annotations

import os
import sys
import time
from pathlib import Path
from typing import List, Optional

import launch_soak


def main(argv: Optional[List[str]] = None) -> int:
    args = list(sys.argv[1:] if argv is None else argv)
    os.environ["X0X_NO_PUBSUB_AFTER_DISCOVER"] = "1"

    if "--soak-dir" not in args:
        repo_root = Path(__file__).resolve().parents[1]
        ts = time.strftime("%Y%m%dT%H%M%SZ", time.gmtime())
        args.extend([
            "--soak-dir",
            str(repo_root / "proofs" / f"launch-soak-fixed-roster-{ts}"),
        ])

    return launch_soak.main(args)


if __name__ == "__main__":
    sys.exit(main())
