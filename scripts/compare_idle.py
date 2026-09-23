#!/usr/bin/env python3
"""Compare idle gallery processes after both release binaries are built.

Usage:
  cargo build --release
  (cd ../gpui-omarchy && cargo build --release --example gallery)
  python3 scripts/compare_idle.py --page virtual_list --runs 3

This measures resident memory and process CPU after the windows settle. It
does not measure rendering frame time or interactive scrolling throughput.
"""

import argparse
import json
import os
import platform
import statistics
import subprocess
import time
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def sample_process(pid: int) -> tuple[int, float]:
    output = subprocess.check_output(
        ["ps", "-p", str(pid), "-o", "rss=", "-o", "%cpu="], text=True
    ).strip()
    memory_kib, cpu_percent = output.split()
    return int(memory_kib), float(cpu_percent)


def measure(binary: Path, page: str, slint: bool, settle: float, samples: int) -> dict:
    command = [str(binary), f"--page={page}" if slint else page]
    started = time.monotonic()
    environment = os.environ.copy()
    if slint:
        environment["SLINT_BACKEND"] = "winit-femtovg"
    process = subprocess.Popen(
        command,
        env=environment,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.PIPE,
        text=True,
    )
    try:
        time.sleep(settle)
        if process.poll() is not None:
            error = process.stderr.read().strip()
            raise RuntimeError(f"{' '.join(command)} exited: {error}")
        readings = []
        for _ in range(samples):
            readings.append(sample_process(process.pid))
            time.sleep(1)
        return {
            "pid": process.pid,
            "rss_mib": round(statistics.median(row[0] for row in readings) / 1024, 2),
            "cpu_percent": round(statistics.median(row[1] for row in readings), 2),
            "elapsed_seconds": round(time.monotonic() - started, 2),
        }
    finally:
        if process.poll() is None:
            process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()
        process.stderr.close()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--page", default="virtual_list")
    parser.add_argument("--runs", type=int, default=3)
    parser.add_argument("--settle", type=float, default=5.0)
    parser.add_argument("--samples", type=int, default=3)
    args = parser.parse_args()
    if args.runs < 1 or args.samples < 1 or args.settle < 0:
        parser.error("runs and samples must be positive, settle must be nonnegative")

    binaries = {
        "slint": ROOT / "target/release/slint-omarchy",
        "gpui": ROOT.parent / "gpui-omarchy/target/release/examples/gallery",
    }
    for binary in binaries.values():
        if not binary.is_file():
            parser.error(f"missing binary: {binary}")

    results = {name: [] for name in binaries}
    for run_index in range(args.runs):
        names = list(binaries)
        if run_index % 2:
            names.reverse()
        for name in names:
            binary = binaries[name]
            results[name].append(
                measure(binary, args.page, name == "slint", args.settle, args.samples)
            )

    print(
        json.dumps(
            {
                "page": args.page,
                "window_px": [1060, 760],
                "platform": platform.platform(),
                "processor": platform.machine(),
                "renderers": {"slint": "winit-femtovg", "gpui": "Metal"},
                "settle_seconds": args.settle,
                "samples_per_run": args.samples,
                "binary_size_mib": {
                    name: round(binary.stat().st_size / 1024 / 1024, 2)
                    for name, binary in binaries.items()
                },
                "runs": results,
                "median_rss_mib": {
                    name: round(statistics.median(run["rss_mib"] for run in runs), 2)
                    for name, runs in results.items()
                },
                "median_cpu_percent": {
                    name: round(statistics.median(run["cpu_percent"] for run in runs), 2)
                    for name, runs in results.items()
                },
            },
            indent=2,
        )
    )


if __name__ == "__main__":
    main()
