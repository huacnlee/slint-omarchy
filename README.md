# slint-omarchy

[slint-omarchy](https://github.com/huacnlee/slint-omarchy) is a Slint implementation of the [gpui-omarchy](https://github.com/huacnlee/gpui-omarchy) component gallery. Its component patterns and examples are based on [GPUI Kit](https://github.com/longbridge/gpui-kit) and gpui-omarchy. This work in progress is intended to compare Slint's development experience and runtime performance with GPUI.

Run the desktop gallery:

```sh
cargo run
```

## Reuse the components

The gallery imports its controls from [`ui/omarchy.slint`](ui/omarchy.slint). That file re-exports controls for actions, forms, navigation, content, data views, overlays, feedback, and the title bar. An application can import the same components and connect their state with Slint bindings. Keep the `ui/` files and `assets/icons/` directory together so the relative imports and icon paths resolve.

```slint
import { Palette, OmButton, OmCheckBox, OmSlider } from "path/to/omarchy.slint";

export component Settings inherits Window {
    background: Palette.background;
    in-out property <bool> sync-enabled: false;
    in-out property <int> volume: 40;

    VerticalLayout {
        OmCheckBox { label: "Sync automatically"; checked <=> root.sync-enabled; }
        OmSlider { label: "Volume"; value <=> root.volume; }
        OmButton { label: "Apply"; primary: true; clicked => { /* save settings */ } }
    }
}
```

[`examples/reuse.rs`](examples/reuse.rs) is a standalone window that imports the component library. Verify the public Slint API with `cargo check --example reuse`.

The host supplies models and handles callbacks for controls such as `OmTable`, `OmTree`, `OmColorPicker`, `OmToast`, and `OmTitleBar`. The gallery in [`ui/gallery.slint`](ui/gallery.slint) shows those bindings in context.

To open the 1,000-row scrolling preview directly:

```sh
cargo run -- --page=virtual_list
```

The software renderer can capture a preview without opening a desktop window. For example:

```sh
cargo run -- --page=accordion --click=350,250 --snapshot=/tmp/slint-accordion.png
```

Repeat `--click=x,y` to exercise a sequence, or use `--drag=x1,y1,x2,y2`, `--type=text`, `--key=Down`/`--key=Return`, and `--wait-ms=6200` for pointer, typing, keyboard, and timer checks. The default capture canvas is 1060 × 760 pixels. Use `--snapshot-size=680x760` to inspect the minimum-width layout.

Slint's built-in performance overlay can be enabled with `SLINT_DEBUG_PERFORMANCE=refresh_full_speed,overlay` for manual frame inspection. Both desktop galleries now open at 1060 × 760 pixels. To compare stable idle process memory and CPU on the same page, build both release binaries and run:

```sh
cargo build --release
(cd ../gpui-omarchy && cargo build --release --example gallery)
python3 scripts/compare_idle.py --page virtual_list --runs 3
```

The script launches the binaries directly, forces Slint's `winit-femtovg` renderer, waits for them to settle, samples each process with `ps`, and reports median resident memory. GPUI uses Metal, so these are end-to-end application figures with different rendering stacks. The script does not measure frame rate, scrolling latency, or startup time. Keep display scale, theme, power mode, and background load the same for both runs.

The [idle comparison](docs/performance.md) records three-run `virtual_list` results for matched event content and row heights, along with its limits. The list's First event and Last event buttons let you inspect the two ends without a long manual scroll.

The sidebar includes previews for 46 reference entries. Dock and OTP input are deferred because their current implementations are incomplete. Some previews cover fewer states than their GPUI counterparts. Use Up/Down, `j`/`k`, or Home/End while the sidebar has focus. Tab reaches buttons and fields. The frameless title bar supports native window dragging, resizing, close, minimize, and maximize controls. Its zoom buttons adjust the gallery from 50% to 200%, and its application menu can follow the current Omarchy theme or preview Tokyo Night and Flexoki Light. System mode checks for theme changes every two seconds.

The system theme is read from `$HOME/.local/state/omarchy/current/theme/colors.toml`, falling back to the legacy `$HOME/.config/omarchy/current` only if the state entry is absent. Invalid themes fall back as a whole to Tokyo Night. Both ANSI and semantic color formats are accepted.

Validation:

```sh
cargo fmt --check
cargo check
cargo test
```

See [design notes](docs/design.md) for the remaining scope and [performance notes](docs/performance.md) for the current idle comparison.
