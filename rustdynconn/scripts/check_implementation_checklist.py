from __future__ import annotations

from pathlib import Path

REQUIRED_MARKERS = [
    "CI-MARKER: lint-rust",
    "CI-MARKER: test-rust",
    "CI-MARKER: test-python",
    "CI-MARKER: wheels-build",
    "CI-MARKER: wheels-smoke",
    "CI-MARKER: release",
    "TEST-MARKER: rust-ett-unit",
    "TEST-MARKER: rust-hdt-targeted",
    "TEST-MARKER: rust-property",
    "TEST-MARKER: python-deterministic",
    "TEST-MARKER: python-random",
    "TEST-MARKER: perf-smoke",
]


def main() -> int:
    checklist_path = Path(__file__).resolve().parents[2] / "IMPLEMENTATION_CHECKLIST.md"
    if not checklist_path.exists():
        print(f"Missing checklist: {checklist_path}")
        return 1

    content = checklist_path.read_text(encoding="utf-8")
    missing = [marker for marker in REQUIRED_MARKERS if marker not in content]
    if missing:
        print("Missing required checklist markers:")
        for marker in missing:
            print(f"- {marker}")
        return 1

    print("Implementation checklist markers present.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
