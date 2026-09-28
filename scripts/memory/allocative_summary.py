#!/usr/bin/env python3
"""Summarize bounded Allocative folded-stack snapshots as JSON."""

from __future__ import annotations

import argparse
import json
from collections import defaultdict
from pathlib import Path


def summarize(path: Path, limit: int) -> dict[str, object]:
    totals: dict[str, int] = defaultdict(int)
    areas: dict[str, int] = defaultdict(int)
    total_bytes = 0

    with path.open(encoding="utf-8") as snapshot:
        for line_number, line in enumerate(snapshot, 1):
            line = line.rstrip("\n")
            if not line:
                continue
            try:
                folded_path, raw_bytes = line.rsplit(" ", 1)
                size = int(raw_bytes)
            except ValueError as error:
                raise ValueError(f"{path}:{line_number}: malformed Allocative line") from error
            totals[folded_path] += size
            total_bytes += size
            parts = folded_path.split(";")
            area = parts[1] if len(parts) > 1 else parts[0]
            areas[area] += size

    return {
        "snapshot": path.name,
        "total_bytes": total_bytes,
        "top_areas": [
            {"name": name, "bytes": size}
            for name, size in sorted(areas.items(), key=lambda item: item[1], reverse=True)[:limit]
        ],
        "top_paths": [
            {"path": name, "bytes": size}
            for name, size in sorted(totals.items(), key=lambda item: item[1], reverse=True)[:limit]
        ],
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("paths", nargs="+", type=Path, help="Allocative .allocative snapshots")
    parser.add_argument("--limit", type=int, default=20, help="Top areas and paths to retain")
    parser.add_argument("--output", type=Path, help="Write JSON to this file instead of stdout")
    args = parser.parse_args()
    if args.limit < 1:
        parser.error("--limit must be at least 1")

    result = {
        "schema_version": 1,
        "kind": "allocative-summary",
        "snapshots": [summarize(path, args.limit) for path in args.paths],
        "limitations": [
            "Allocative traverses selected roots only; this summary is not a complete heap census.",
            "This report aggregates retained object paths and does not contain allocation events or source contents.",
        ],
    }
    rendered = json.dumps(result, ensure_ascii=False, indent=2) + "\n"
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered, encoding="utf-8")
    else:
        print(rendered, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
