#!/usr/bin/env python3
"""Unit tests for the pure helpers in adr-governance.py.

The validator itself is git-driven and exercised end-to-end by the ADR
Governance workflow. These tests cover the template-repair allowance
(is_template_repair) in isolation, since that logic decides whether an edit
to an Accepted ADR is a permitted heading-only conformance fix or a
forbidden mutation — a distinction worth pinning down with explicit cases.

Run: python3 scripts/test_adr_governance.py
"""
from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

_spec = importlib.util.spec_from_file_location(
    "adr_governance", Path(__file__).with_name("adr-governance.py")
)
assert _spec and _spec.loader
adr = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(adr)


# A minimal Accepted ADR whose Validation section is mistitled, mirroring the
# real 0016 slip ("## Acceptance criteria" in the Validation slot).
_MISTITLED = """\
- Status: Accepted (2026-06-11)

## Context
Some context.

## Decision
The decision body.

## Consequences
The consequences.

## Acceptance criteria
- criterion one
- criterion two
"""

_REPAIRED = _MISTITLED.replace("## Acceptance criteria", "## Validation")


def check(name: str, cond: bool) -> bool:
    print(f"{'ok' if cond else 'FAIL'} - {name}")
    return cond


def main() -> int:
    results = [
        # The real case: renaming the mistitled heading is allowed.
        check(
            "heading-only rename restoring a required section is a repair",
            adr.is_template_repair(_MISTITLED, _REPAIRED),
        ),
        # No required section was missing -> nothing to repair, not allowed.
        check(
            "no-op edit on an already-compliant ADR is not a repair",
            not adr.is_template_repair(_REPAIRED, _REPAIRED),
        ),
        # Touching body content (even while adding the section) is blocked.
        check(
            "editing decision body is not a repair",
            not adr.is_template_repair(
                _MISTITLED,
                _REPAIRED.replace("The decision body.", "A reworded decision."),
            ),
        ),
        # Adding a brand-new section with new prose changes non-heading lines,
        # so it falls through to immutability (must supersede instead).
        check(
            "adding a new section with new content is not a repair",
            not adr.is_template_repair(
                _MISTITLED.replace("## Acceptance criteria\n- criterion one\n- criterion two\n", ""),
                _MISTITLED.replace("## Acceptance criteria", "## Validation"),
            ),
        ),
        # The result must end up fully compliant; renaming to a non-required
        # heading does not satisfy the required-section set.
        check(
            "rename that does not restore a required section is not a repair",
            not adr.is_template_repair(
                _MISTITLED, _MISTITLED.replace("## Acceptance criteria", "## Notes")
            ),
        ),
        check("has_section detects present heading", adr.has_section(_REPAIRED, "Validation")),
        check("has_section rejects absent heading", not adr.has_section(_MISTITLED, "Validation")),
    ]
    failed = results.count(False)
    print(f"\n{len(results) - failed}/{len(results)} passed")
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
