#!/usr/bin/env python3
"""Compare three-build Rspack memory samples using the local acceptance gate."""

from __future__ import annotations

import argparse
import json
import statistics
import sys
from pathlib import Path
from typing import Any


def load_build_samples(directory: Path, require_rust: bool) -> list[dict[str, Any]]:
    samples = []
    for path in sorted(directory.glob("*.json")):
        try:
            report = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError):
            continue
        if report.get("phase") != "build":
            continue
        expected_allocator = "jemalloc" if require_rust else "default"
        if report.get("allocator") != expected_allocator:
            continue
        process = report.get("process") or {}
        sample = {
            "file": path.name,
            "timestamp": report.get("timestamp", ""),
            "process_id": report.get("process_id"),
            "process_start_timestamp": report.get("process_start_timestamp"),
            "duration_ms": report.get("duration_ms"),
            "peak_rss_bytes": process.get("peak_rss_bytes"),
            "process_peak_rss_bytes": process.get("process_peak_rss_bytes"),
            "rust_live_bytes": report.get("rust_live_bytes"),
        }
        required = [
            sample["duration_ms"],
            sample["peak_rss_bytes"],
            sample["process_peak_rss_bytes"],
        ]
        if require_rust:
            required.append(sample["rust_live_bytes"])
        if any(not isinstance(value, (int, float)) or value <= 0 for value in required):
            continue
        samples.append(sample)
    samples.sort(key=lambda item: item["timestamp"])
    # Multi-compiler processes can emit one report per compiler. Keep one cold-run sample
    # per Node process, using the last live-byte reading and the maximum observed peaks.
    by_process = {}
    for sample in samples:
        process_id = sample["process_id"]
        process_start = sample["process_start_timestamp"]
        key = (
            (process_id, process_start)
            if process_id is not None and process_start is not None
            else process_id if process_id is not None else sample["file"]
        )
        previous = by_process.get(key)
        if previous is None:
            by_process[key] = sample
            continue
        by_process[key] = {
            **sample,
            "peak_rss_bytes": max(previous["peak_rss_bytes"], sample["peak_rss_bytes"]),
            "process_peak_rss_bytes": max(
                previous["process_peak_rss_bytes"], sample["process_peak_rss_bytes"]
            ),
            "duration_ms": max(previous["duration_ms"], sample["duration_ms"]),
        }
    samples = sorted(by_process.values(), key=lambda item: item["timestamp"])
    if len(samples) < 3:
        label = "Rust profiler" if require_rust else "default allocator"
        raise ValueError(f"{directory}: need at least 3 distinct process build samples from the {label}, found {len(samples)}")
    return samples[-3:]


def median(samples: list[dict[str, Any]], key: str) -> float:
    return float(statistics.median(sample[key] for sample in samples))


def reduction(before: float, after: float) -> float:
    return (before - after) / before if before else 0.0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--baseline-rust", required=True, type=Path)
    parser.add_argument("--candidate-rust", required=True, type=Path)
    parser.add_argument("--baseline-process", required=True, type=Path)
    parser.add_argument("--candidate-process", required=True, type=Path)
    parser.add_argument("--minimum-memory-reduction", type=float, default=0.05)
    parser.add_argument("--maximum-duration-regression", type=float, default=0.05)
    args = parser.parse_args()

    try:
        baseline_rust = load_build_samples(args.baseline_rust, require_rust=True)
        candidate_rust = load_build_samples(args.candidate_rust, require_rust=True)
        baseline_process = load_build_samples(args.baseline_process, require_rust=False)
        candidate_process = load_build_samples(args.candidate_process, require_rust=False)
    except ValueError as error:
        print(json.dumps({"accepted": False, "error": str(error)}, indent=2))
        return 2

    rust_before = median(baseline_rust, "rust_live_bytes")
    rust_after = median(candidate_rust, "rust_live_bytes")
    rss_before = median(baseline_process, "process_peak_rss_bytes")
    rss_after = median(candidate_process, "process_peak_rss_bytes")
    duration_before = median(baseline_process, "duration_ms")
    duration_after = median(candidate_process, "duration_ms")
    rust_reduction = reduction(rust_before, rust_after)
    rss_reduction = reduction(rss_before, rss_after)
    duration_regression = (duration_after - duration_before) / duration_before

    checks = {
        "rust_live_bytes_reduction": rust_reduction >= args.minimum_memory_reduction,
        "default_allocator_process_peak_rss_reduction": rss_reduction >= args.minimum_memory_reduction,
        "duration_regression_within_limit": duration_regression <= args.maximum_duration_regression,
    }
    result = {
        "schema_version": 1,
        "accepted": all(checks.values()),
        "thresholds": {
            "minimum_memory_reduction": args.minimum_memory_reduction,
            "maximum_duration_regression": args.maximum_duration_regression,
            "samples_per_series": 3,
        },
        "medians": {
            "baseline_rust_live_bytes": rust_before,
            "candidate_rust_live_bytes": rust_after,
            "rust_live_bytes_reduction": rust_reduction,
            "baseline_default_process_peak_rss_bytes": rss_before,
            "candidate_default_process_peak_rss_bytes": rss_after,
            "default_process_peak_rss_reduction": rss_reduction,
            "baseline_compile_duration_ms": duration_before,
            "candidate_compile_duration_ms": duration_after,
            "compile_duration_regression": duration_regression,
        },
        "checks": checks,
        "sample_files": {
            "baseline_rust": [sample["file"] for sample in baseline_rust],
            "candidate_rust": [sample["file"] for sample in candidate_rust],
            "baseline_process": [sample["file"] for sample in baseline_process],
            "candidate_process": [sample["file"] for sample in candidate_process],
        },
        "measurement_note": "Rust live bytes must come from the jemalloc profiling build; RSS and duration must come from the default allocator build.",
    }
    print(json.dumps(result, indent=2))
    return 0 if result["accepted"] else 1


if __name__ == "__main__":
    sys.exit(main())
