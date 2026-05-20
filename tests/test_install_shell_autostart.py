#!/usr/bin/env python3
"""Focused regression tests for scripts/install.sh."""

from __future__ import annotations

import os
import subprocess
import tarfile
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
INSTALL_SH = ROOT / "scripts" / "install.sh"
PLATFORM = "linux-x64-gnu"


def write_executable(path: Path, contents: str) -> None:
    path.write_text(contents, encoding="utf-8")
    path.chmod(0o755)


class InstallShellAutostartTests(unittest.TestCase):
    def test_named_autostart_forwards_name_to_x0x(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            tmp = Path(tmpdir)
            fake_bin = tmp / "fake-bin"
            archive_dir = tmp / "archive" / f"x0x-{PLATFORM}"
            home = tmp / "home"
            data_home = tmp / "data"
            calls = tmp / "x0x-calls.log"
            archive = tmp / f"x0x-{PLATFORM}.tar.gz"

            fake_bin.mkdir()
            archive_dir.mkdir(parents=True)
            home.mkdir()

            write_executable(
                fake_bin / "uname",
                """#!/usr/bin/env sh
case "$1" in
    -s) echo Linux ;;
    -m) echo x86_64 ;;
    *) exit 1 ;;
esac
""",
            )

            write_executable(
                archive_dir / "x0xd",
                """#!/usr/bin/env sh
NAME=""
while [ $# -gt 0 ]; do
    case "$1" in
        --name) shift; NAME="$1" ;;
    esac
    shift
done
if [ -n "$NAME" ]; then
    DIR="${XDG_DATA_HOME:-$HOME/.local/share}/x0x-$NAME"
else
    DIR="${XDG_DATA_HOME:-$HOME/.local/share}/x0x"
fi
mkdir -p "$DIR"
printf '%s\\n' "127.0.0.1:65535" > "$DIR/api.port"
""",
            )

            write_executable(
                archive_dir / "x0x",
                """#!/usr/bin/env sh
printf '%s\\n' "$*" >> "$XOX_CALLS"
""",
            )

            with tarfile.open(archive, "w:gz") as tar:
                tar.add(archive_dir / "x0xd", arcname=f"x0x-{PLATFORM}/x0xd")
                tar.add(archive_dir / "x0x", arcname=f"x0x-{PLATFORM}/x0x")

            write_executable(
                fake_bin / "curl",
                f"""#!/usr/bin/env sh
OUT=""
URL=""
while [ $# -gt 0 ]; do
    case "$1" in
        -o) shift; OUT="$1" ;;
        http*) URL="$1" ;;
    esac
    shift
done
if [ -n "$OUT" ]; then
    cp "{archive}" "$OUT"
    exit 0
fi
case "$URL" in
    */health) printf '%s\\n' '{{"ok":true}}' ;;
    */agent) printf '%s\\n' '{{}}' ;;
    *) exit 1 ;;
esac
""",
            )

            env = os.environ.copy()
            env.update(
                {
                    "HOME": str(home),
                    "XDG_DATA_HOME": str(data_home),
                    "PATH": f"{fake_bin}{os.pathsep}{env.get('PATH', '')}",
                    "TMPDIR": str(tmp),
                    "XOX_CALLS": str(calls),
                }
            )

            result = subprocess.run(
                ["sh", str(INSTALL_SH), "--name", "alice", "--autostart"],
                cwd=ROOT,
                env=env,
                text=True,
                capture_output=True,
                timeout=20,
            )

            self.assertEqual(result.returncode, 0, result.stderr + result.stdout)
            self.assertEqual(
                calls.read_text(encoding="utf-8").splitlines(),
                ["--name alice autostart"],
            )

    def test_unhealthy_daemon_fails_install(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            tmp = Path(tmpdir)
            fake_bin = tmp / "fake-bin"
            archive_dir = tmp / "archive" / f"x0x-{PLATFORM}"
            home = tmp / "home"
            data_home = tmp / "data"
            archive = tmp / f"x0x-{PLATFORM}.tar.gz"

            fake_bin.mkdir()
            archive_dir.mkdir(parents=True)
            home.mkdir()

            write_executable(
                fake_bin / "uname",
                """#!/usr/bin/env sh
case "$1" in
    -s) echo Linux ;;
    -m) echo x86_64 ;;
    *) exit 1 ;;
esac
""",
            )

            write_executable(
                fake_bin / "sleep",
                """#!/usr/bin/env sh
python3 -c 'import time; time.sleep(0.01)'
""",
            )

            write_executable(
                archive_dir / "x0xd",
                """#!/usr/bin/env sh
DIR="${XDG_DATA_HOME:-$HOME/.local/share}/x0x"
mkdir -p "$DIR"
printf '%s\\n' "127.0.0.1:65535" > "$DIR/api.port"
""",
            )

            write_executable(
                archive_dir / "x0x",
                """#!/usr/bin/env sh
exit 0
""",
            )

            with tarfile.open(archive, "w:gz") as tar:
                tar.add(archive_dir / "x0xd", arcname=f"x0x-{PLATFORM}/x0xd")
                tar.add(archive_dir / "x0x", arcname=f"x0x-{PLATFORM}/x0x")

            write_executable(
                fake_bin / "curl",
                f"""#!/usr/bin/env sh
OUT=""
while [ $# -gt 0 ]; do
    case "$1" in
        -o) shift; OUT="$1" ;;
    esac
    shift
done
if [ -n "$OUT" ]; then
    cp "{archive}" "$OUT"
    exit 0
fi
exit 7
""",
            )

            env = os.environ.copy()
            env.update(
                {
                    "HOME": str(home),
                    "XDG_DATA_HOME": str(data_home),
                    "PATH": f"{fake_bin}{os.pathsep}{env.get('PATH', '')}",
                    "TMPDIR": str(tmp),
                }
            )

            result = subprocess.run(
                ["sh", str(INSTALL_SH)],
                cwd=ROOT,
                env=env,
                text=True,
                capture_output=True,
                timeout=20,
            )

            output = result.stderr + result.stdout
            self.assertNotEqual(result.returncode, 0, output)
            self.assertIn("Timeout waiting for healthy daemon", output)
            self.assertNotIn("x0x is running", output)


if __name__ == "__main__":
    unittest.main()
