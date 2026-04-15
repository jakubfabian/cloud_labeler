#!/usr/bin/env python3
"""
Parse Criterion (Rust) and Fortran benchmark outputs into the
github-action-benchmark `customSmallerIsBetter` JSON format.

Output format:
  [{"name": str, "unit": "ns", "value": float}, ...]

Usage:
  python3 scripts/parse_benchmarks.py \
    --criterion-dir rust/target/criterion \
    --fortran-log   /tmp/fortran_bench.txt \
    --output        /tmp/benchmark_results.json
"""

import argparse
import json
import os
import re
import sys


# ── Criterion parser ───────────────────────────────────────────────────────────

def parse_criterion(criterion_dir: str) -> list[dict]:
    """
    Walk `criterion_dir` and collect all `new/estimates.json` files.

    Directory layout:
        <criterion_dir>/<group>/<bench>/new/estimates.json

    The `mean.point_estimate` field is already in nanoseconds.
    Returns entries named `rust/<group>/<bench>`.
    """
    results = []
    if not os.path.isdir(criterion_dir):
        print(f"[warn] Criterion dir not found: {criterion_dir}", file=sys.stderr)
        return results

    for group in sorted(os.listdir(criterion_dir)):
        group_dir = os.path.join(criterion_dir, group)
        if not os.path.isdir(group_dir):
            continue
        # Skip the top-level `report` directory that Criterion creates
        if group == "report":
            continue

        for bench in sorted(os.listdir(group_dir)):
            estimates_path = os.path.join(group_dir, bench, "new", "estimates.json")
            if not os.path.isfile(estimates_path):
                continue

            with open(estimates_path) as fh:
                data = json.load(fh)

            value_ns = data["mean"]["point_estimate"]
            results.append({
                "name": f"rust/{group}/{bench}",
                "unit": "ns",
                "value": round(value_ns, 3),
            })

    return results


# ── Fortran parser ─────────────────────────────────────────────────────────────

# Matches lines like:
#   cross_10x10x1   :    364.1 ns/iter  (100000 reps)
#   cross_100x100x1 :     18.234 µs/iter  (10000 reps)
#   cross_50x50x50  :    234.567 us/iter  (2000 reps)
_FORT_PATTERN = re.compile(
    r"(cross_\S+)\s*:\s*([\d.]+)\s*(ns|us|\xb5s|\u03bcs)/iter",
    re.IGNORECASE,
)


def parse_fortran(log_path: str) -> list[dict]:
    """
    Parse Fortran bench output and return entries in nanoseconds.
    Both `ns` and `µs` / `us` unit suffixes are handled.
    Returns entries named `fortran/<bench>`.
    """
    results = []
    if not os.path.isfile(log_path):
        print(f"[warn] Fortran log not found: {log_path}", file=sys.stderr)
        return results

    with open(log_path, encoding="utf-8", errors="replace") as fh:
        for line in fh:
            m = _FORT_PATTERN.search(line)
            if not m:
                continue

            name_raw, value_str, unit = m.group(1), m.group(2), m.group(3).lower()
            value = float(value_str)

            # Normalise to nanoseconds
            if unit in ("us", "\xb5s", "\u03bcs"):   # µs
                value_ns = value * 1_000.0
            else:                                      # ns
                value_ns = value

            results.append({
                "name": f"fortran/{name_raw.strip('_')}",
                "unit": "ns",
                "value": round(value_ns, 3),
            })

    return results


# ── Main ───────────────────────────────────────────────────────────────────────

def main() -> None:
    parser = argparse.ArgumentParser(
        description="Merge Criterion + Fortran benchmark results into customSmallerIsBetter JSON"
    )
    parser.add_argument("--criterion-dir", default="rust/target/criterion",
                        help="Root of Criterion output (default: rust/target/criterion)")
    parser.add_argument("--fortran-log", default="",
                        help="Path to Fortran bench output text file")
    parser.add_argument("--output", default="-",
                        help="Output JSON file path, or - for stdout (default: -)")
    args = parser.parse_args()

    results: list[dict] = []
    results.extend(parse_criterion(args.criterion_dir))
    if args.fortran_log:
        results.extend(parse_fortran(args.fortran_log))

    if not results:
        print("[warn] No benchmark results found — output will be empty.", file=sys.stderr)

    payload = json.dumps(results, indent=2)

    if args.output == "-":
        print(payload)
    else:
        with open(args.output, "w") as fh:
            fh.write(payload)
        print(f"Wrote {len(results)} entries to {args.output}", file=sys.stderr)


if __name__ == "__main__":
    main()
