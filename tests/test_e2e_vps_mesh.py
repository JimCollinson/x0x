#!/usr/bin/env python3
"""Focused tests for the VPS mesh harness split-soak controls."""

from __future__ import annotations

import base64
import importlib.util
import json
import logging
import sys
import unittest
from pathlib import Path


def load_mesh():
    script = Path(__file__).with_name("e2e_vps_mesh.py")
    spec = importlib.util.spec_from_file_location("e2e_vps_mesh", script)
    assert spec is not None
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


class FakeClient:
    def __init__(self) -> None:
        self.published: list[tuple[str, bytes]] = []
        self.direct_sent: list[tuple[str, bytes]] = []

    def publish(self, topic: str, payload: bytes) -> None:
        self.published.append((topic, payload))

    def direct_send(self, target_aid: str, payload: bytes, **_kwargs):
        self.direct_sent.append((target_aid, payload))
        return {"ok": True, "via": "direct"}


class E2eVpsMeshTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.mesh = load_mesh()

    def test_publish_discover_marks_no_pubsub_after_discover(self) -> None:
        client = FakeClient()

        self.mesh.publish_discover(
            client,
            "a" * 64,
            "request-1",
            no_pubsub_after_discover=True,
        )

        self.assertEqual(self.mesh.DISCOVER_TOPIC, client.published[0][0])
        payload = json.loads(client.published[0][1])
        self.assertTrue(payload["no_pubsub_after_discover"])
        self.assertTrue(payload["params"]["no_pubsub_after_discover"])

    def test_anchor_local_command_can_avoid_pubsub_fallback(self) -> None:
        client = FakeClient()
        command = {"action": "noop_ack", "params": {}}
        anchor = "a" * 64

        result = self.mesh.send_command_dm(
            client,
            anchor,
            command,
            logging.getLogger("test"),
            anchor_aid=anchor,
            allow_anchor_pubsub=False,
        )

        self.assertEqual({"ok": True, "via": "direct"}, result)
        self.assertEqual([], client.published)
        self.assertEqual(anchor, client.direct_sent[0][0])
        wire = client.direct_sent[0][1]
        self.assertTrue(wire.startswith(self.mesh.PREFIX_CMD))
        decoded = json.loads(base64.b64decode(wire[len(self.mesh.PREFIX_CMD):]))
        self.assertEqual(command, decoded)


if __name__ == "__main__":
    unittest.main()
