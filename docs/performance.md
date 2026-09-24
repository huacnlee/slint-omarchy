# Idle gallery comparison · 2026-09-23

Historical baseline: this run used Slint 1.16.0. The project now uses Slint 1.18.1, so these numbers do not describe the current build.

The current comparison uses the 1,000-row `virtual_list` page in both release galleries. The Slint example now matches the GPUI example's event text, one-in-five expanded rows, and 44/28 px row heights. Both windows are 1060 × 760 pixels. The [sampling script](../scripts/compare_idle.py) launched each binary three times, waited five seconds, then collected three one-second-spaced `ps` readings per run. It alternated launch order. [Raw readings](bench-virtual-list-idle-2026-09-23-parity.json) are retained.

| Measure | Slint 1.16.0 | GPUI Omarchy `94705f0` |
| --- | ---: | ---: |
| Median resident memory | 96.55 MiB | 85.52 MiB |
| Release binary size | 26.09 MiB | 22.17 MiB |

Environment: Mac15,3, arm64, 24 GiB RAM, macOS 27.0. Slint used `winit-femtovg`; GPUI used Metal. The same Omarchy theme loader and 1,000-row preview were used, but component implementations and rendering stacks differ. These values describe the complete applications, not isolated framework overhead. CPU readings varied substantially between runs and are only included in the raw JSON.

This run does not measure startup time, scrolling frame time, input latency, peak memory, or battery use. Those need a separate instrumented pass with visible-window and interaction verification. The current desktop capture path returns black frames, so visual rendering during these launches has not been confirmed from screenshots. The [earlier readings](bench-virtual-list-idle-2026-09-23.json) used different Slint row content and sizes and should not be compared directly with this run.
