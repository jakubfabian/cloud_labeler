#!/usr/bin/env python3
"""
Benchmark: Fortran (f2py) vs Rust (PyO3) cloud_labeler.

Run from the CMake build directory:
    python python/benchmark.py

Or pass custom sizes:
    python python/benchmark.py --sizes 10x10x1 100x100x50 --reps 500
"""

import sys
import os
import time
import argparse
import numpy as np

# ── Module discovery ──────────────────────────────────────────────────────────
# Both .so files are built into the same directory as this script.
_here = os.path.dirname(os.path.abspath(__file__))
if _here not in sys.path:
    sys.path.insert(0, _here)

try:
    import py_cloud_label as _fort_mod
    # f2py wraps the Fortran module m_py_cloud_label inside the extension.
    # The py_gen_labels subroutine is at _fort_mod.m_py_cloud_label.py_gen_labels.
    _fort = _fort_mod.m_py_cloud_label
    HAS_FORTRAN = True
except ImportError as e:
    print(f"[warn] Fortran module unavailable: {e}", file=sys.stderr)
    HAS_FORTRAN = False

try:
    import cloud_labeler_rs as _rust
    HAS_RUST = True
except ImportError as e:
    print(f"[warn] Rust module unavailable: {e}", file=sys.stderr)
    HAS_RUST = False

if not HAS_FORTRAN and not HAS_RUST:
    sys.exit("Neither module is available — build with -DENABLE_PYTHON=YES first.")

# ── Test-field builders ───────────────────────────────────────────────────────

def build_cross(nx: int, ny: int, nz: int) -> np.ndarray:
    """Cross-shaped cloud field — mirrors Fortran test.f90."""
    cld = np.zeros((nx, ny, nz), dtype=bool, order="F")
    cld[nx // 2, 1 : ny - 1, :] = True
    cld[1 : nx - 1, ny // 2, :] = True
    return cld


def build_random(nx: int, ny: int, nz: int, density: float = 0.3, seed: int = 42) -> np.ndarray:
    """Random cloud field at the given density."""
    rng = np.random.default_rng(seed)
    return np.asfortranarray(rng.random((nx, ny, nz)) < density)


# ── Wrappers ──────────────────────────────────────────────────────────────────

def run_fortran(cld: np.ndarray) -> np.ndarray:
    """Call the Fortran f2py gen_labels wrapper.
    nx/ny/nz are optional in the f2py signature — inferred from cld.shape."""
    return _fort.py_gen_labels(cld)


def run_rust(cld: np.ndarray) -> np.ndarray:
    """Call the Rust PyO3 gen_labels wrapper."""
    return _rust.gen_labels(cld)


# ── Correctness check ─────────────────────────────────────────────────────────

def check_agreement(cld: np.ndarray, name: str) -> bool:
    """Verify Fortran and Rust return the same number of patches."""
    if not (HAS_FORTRAN and HAS_RUST):
        return True
    fl = run_fortran(cld)
    rl = run_rust(cld)
    n_fort = int(fl.max()) + 1 if fl.max() >= 0 else 0
    n_rust = int(rl.max()) + 1 if rl.max() >= 0 else 0
    ok = n_fort == n_rust
    status = "OK" if ok else "MISMATCH"
    print(f"  [{status}] {name}: Fortran={n_fort} patches, Rust={n_rust} patches")
    return ok


# ── Timing ────────────────────────────────────────────────────────────────────

def measure(fn, cld: np.ndarray, reps: int) -> float:
    """Return mean wall-clock time per call (seconds)."""
    fn(cld)  # warmup
    t0 = time.perf_counter()
    for _ in range(reps):
        fn(cld)
    return (time.perf_counter() - t0) / reps


def fmt_time(seconds: float) -> str:
    if seconds < 1e-6:
        return f"{seconds * 1e9:7.1f} ns"
    if seconds < 1e-3:
        return f"{seconds * 1e6:7.3f} µs"
    if seconds < 1.0:
        return f"{seconds * 1e3:7.3f} ms"
    return f"{seconds:7.3f}  s"


# ── Main ──────────────────────────────────────────────────────────────────────

DEFAULT_SIZES = ["10x10x1", "100x100x1", "50x50x50", "128x128x32"]
DEFAULT_REPS  = {"10x10x1": 5000, "100x100x1": 500, "50x50x50": 100, "128x128x32": 20}


def parse_size(s: str):
    parts = s.split("x")
    if len(parts) != 3:
        raise argparse.ArgumentTypeError(f"expected NxNxN, got {s!r}")
    return tuple(int(p) for p in parts)


def run(sizes, reps_override, field, check_only=False):
    print()
    print("╔══════════════════════════════════════════════════════════════╗")
    print("║   Cloud Labeler Python Benchmark — Fortran vs Rust           ║")
    print("╚══════════════════════════════════════════════════════════════╝")
    if HAS_FORTRAN: print("  Fortran : available (f2py, m_cloud_label module)")
    else:           print("  Fortran : NOT available")
    if HAS_RUST:    print("  Rust    : available (PyO3, cloud_labeler_rs module)")
    else:           print("  Rust    : NOT available")

    # ── correctness check ──────────────────────────────────────────────────────
    print("\n── Correctness check (cross pattern) ────────────────────────────")
    all_ok = True
    for s in sizes:
        cld = build_cross(*s)
        ok = check_agreement(cld, "x".join(map(str, s)))
        all_ok = all_ok and ok

    if check_only:
        if all_ok:
            print("\nAll correctness checks passed.")
            sys.exit(0)
        else:
            print("\nERROR: correctness check failed.", file=sys.stderr)
            sys.exit(1)

    # ── timing ─────────────────────────────────────────────────────────────────
    print()
    col = 22
    hdr = (f"  {'Workload':<{col}} {'Fortran':>12} {'Rust':>12} "
           f"{'Speedup':>10}  {'Patches':>8}")
    print("── Timing " + "─" * (len(hdr) - 9))
    print(f"  {'field type':<{col}} {'(f2py)':>12} {'(PyO3)':>12} "
          f"{'Rust/Fort':>10}  {'(count)':>8}")
    print("  " + "─" * (len(hdr) - 2))

    for s in sizes:
        nx, ny, nz = s
        key = "x".join(map(str, s))
        reps = reps_override if reps_override else DEFAULT_REPS.get(key, 50)

        cld = build_cross(*s) if field == "cross" else build_random(*s)

        # Count patches (Fortran or Rust, whichever is available)
        if HAS_FORTRAN:
            labels = run_fortran(cld)
            n_patches = int(labels.max()) + 1 if labels.max() >= 0 else 0
        elif HAS_RUST:
            labels = run_rust(cld)
            n_patches = int(labels.max()) + 1 if labels.max() >= 0 else 0
        else:
            n_patches = -1

        ft = measure(run_fortran, cld, reps) if HAS_FORTRAN else None
        rt = measure(run_rust,    cld, reps) if HAS_RUST    else None

        ft_str = fmt_time(ft) if ft else "       N/A"
        rt_str = fmt_time(rt) if rt else "       N/A"
        sp_str = f"{ft/rt:8.2f}×" if (ft and rt) else "       N/A"

        print(f"  {key:<{col}} {ft_str:>12} {rt_str:>12} {sp_str:>10}  {n_patches:>8}")

    print()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Benchmark Fortran f2py vs Rust PyO3 cloud_labeler")
    parser.add_argument(
        "--sizes", nargs="+", default=DEFAULT_SIZES,
        metavar="NxNxN",
        help="Domain sizes, e.g. 10x10x1 100x100x50 (default: %(default)s)")
    parser.add_argument(
        "--reps", type=int, default=None,
        help="Repetitions per size (auto-selected if omitted)")
    parser.add_argument(
        "--field", choices=["cross", "random"], default="cross",
        help="Cloud field pattern (default: cross)")
    parser.add_argument(
        "--check-only", action="store_true",
        help="Run correctness check only (no timing); exit 0 on pass, 1 on fail")
    args = parser.parse_args()

    sizes = [parse_size(s) for s in args.sizes]
    run(sizes, args.reps, args.field, check_only=args.check_only)
