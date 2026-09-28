# Idle gallery comparison · 2026-09-24

The Slint 1.18.1 gallery after the reusable-component and visual fixes and the GPUI Omarchy `94705f0` release gallery were sampled on the `virtual_list` page at 1060 × 760. The [sampling script](../scripts/compare_idle.py) ran each application three times, alternated launch order, waited five seconds, and took three one-second-spaced `ps` samples per run. On macOS it required an on-screen, layer-zero window with the expected dimensions before sampling each process. Window-specific screenshots of both applications were also inspected to confirm rendered content. [Raw readings](bench-virtual-list-idle-2026-09-24-visible-window.json) are retained.

| Measure | Slint 1.18.1 | GPUI Omarchy `94705f0` |
| --- | ---: | ---: |
| Median resident memory | 101.80 MiB | 84.95 MiB |
| Release binary size | 24.28 MiB | 22.17 MiB |

Environment: Mac15,3, arm64, 24 GiB RAM, macOS 27.0. Slint used `winit-femtovg`; GPUI used Metal. These are whole-application idle figures, not framework overhead. CPU samples varied and do not support a reliable CPU comparison. This script does not measure startup time, scrolling latency, frame rate, or power use. The window check confirms presence and dimensions, not equal rendering work or interaction performance.

The [earlier 2026-09-24 readings](bench-virtual-list-idle-2026-09-24-slint-1.18.1.json) were 101.81 MiB for Slint and 85.02 MiB for GPUI. They preceded the current visual changes and did not verify an on-screen window during sampling.

The Slint 1.16.0 baseline below was collected on the previous day. It used the same scenario, but the conditions were not controlled tightly enough to attribute the memory difference to the Slint upgrade alone.

# Idle gallery comparison · 2026-09-23

Historical baseline: this run used Slint 1.16.0. The project now uses Slint 1.18.1, so these numbers do not describe the current build.

The historical comparison used the 1,000-row `virtual_list` page in both release galleries. The Slint example matched the GPUI example's event text, one-in-five expanded rows, and 44/28 px row heights. Both windows are 1060 × 760 pixels. The [sampling script](../scripts/compare_idle.py) launched each binary three times, waited five seconds, then collected three one-second-spaced `ps` readings per run. It alternated launch order. [Raw readings](bench-virtual-list-idle-2026-09-23-parity.json) are retained.

| Measure | Slint 1.16.0 | GPUI Omarchy `94705f0` |
| --- | ---: | ---: |
| Median resident memory | 96.55 MiB | 85.52 MiB |
| Release binary size | 26.09 MiB | 22.17 MiB |

Environment: Mac15,3, arm64, 24 GiB RAM, macOS 27.0. Slint used `winit-femtovg`; GPUI used Metal. The same Omarchy theme loader and 1,000-row preview were used, but component implementations and rendering stacks differ. These values describe the complete applications, not isolated framework overhead. CPU readings varied substantially between runs and are only included in the raw JSON.

This run does not measure startup time, scrolling frame time, input latency, peak memory, or battery use. Those need a separate instrumented pass with visible-window and interaction verification. The current desktop capture path returns black frames, so visual rendering during these launches has not been confirmed from screenshots. The [earlier readings](bench-virtual-list-idle-2026-09-23.json) used different Slint row content and sizes and should not be compared directly with this run.
