# UI module architecture

This project ports the GPUI Omarchy gallery to Slint. The public entry point is
[`ui/omarchy.slint`](../ui/omarchy.slint). A consumer imports controls from that
file; [`ui/gallery.slint`](../ui/gallery.slint) is an application that uses them.

The design follows the ownership rule in [`gpui-kit`'s base architecture](https://github.com/longbridge/gpui-kit/blob/main/docs/ARCHITECTURE.md): reusable behavior and the geometry needed for that behavior belong together; the styled layer owns the visual language; the application owns product data and copy. Slint already supplies layout, text editing, focus scopes, and some scrolling behavior, so this project uses those built-ins directly instead of imitating `gpui-base` with a second crate.

| Owner | Files | Responsibility |
| --- | --- | --- |
| Slint runtime and standard widgets | `std-widgets.slint`, `TextInput`, `FocusScope`, layouts | Text editing, IME, hit testing, focus, layout, and viewport mechanics |
| Theme | `ui/theme.slint` | Semantic palette shared by styled controls; Rust swaps the complete palette |
| Reusable controls | `ui/base.slint`, `ui/actions.slint`, `ui/forms.slint`, `ui/layout.slint`, and other modules exported by `ui/omarchy.slint` | Visual states, stable control bounds, input handling, accessibility, and coupled geometry |
| Gallery application | `ui/gallery.slint`, `ui/gallery_examples.slint`, `src/main.rs` | Example data, copy, navigation, dialog workflow, and host callbacks |

## Public interface rules

- Export a control from `omarchy.slint` only when another application can use it without adopting gallery-specific data or wording. `OmWorkspaceForm`, `OmResetDialogForm`, `OmProjectTable`, and the sample split panes live in `gallery_examples.slint` for this reason.
- `OmTable`, `OmTableRow`, and `OmTableCell` own table geometry, focus, and row state. The caller composes its columns and cells; the five-column project schema is only one gallery adapter. The standalone reuse example composes a two-column service table through the same public interface.
- `OmVirtualTextList` owns viewport scrolling and the matching scrollbar; callers provide text rows, explicit row heights, and their sum as `total-height`. The same extent drives the `ListView`, last-item jump, and custom scrollbar, so variable-height estimates do not clip the last row. Slint 1.18.1 requires a `ListView` to contain a direct `for` repeater and rejects `@children` there, so arbitrary row composition is not part of this public interface.
- `OmRichText` accepts caller-owned `styled-text`, theme colors, and a link callback. Slint 1.18.1 renders inline Markdown, lists, and links through `StyledText`; it does not provide selectable text, keyboard link activation, document headings, images, tables, block quotes, code blocks, or general HTML in that element. The gallery-only `OmTextView` composes a fixed article schema for demonstration while those runtime limits remain.
- `OmHorizontalSplit` and `OmVerticalSplit` own a draggable, keyboard-adjustable divider and size limits. Callers supply both panes through `@children` and bind their position and size to `first-size`, `second-offset`, and `second-size`; sample document copy stays in the gallery.
- Keep complicated behavior behind a small interface. `OmCollapsible` takes a title, a two-way `expanded` value, and child content; it owns its trigger, keyboard activation, panel geometry, and visual state. Callers should not rebuild those details.
- A control's value belongs to the caller. Bind it with `<=>` or handle a callback and update the application model. Ephemeral pointer and focus state can stay inside the control.
- Use semantic theme roles in controls. The gallery can choose sample colors or content, but those choices must not leak into a general control.
- Put arbitrary content in `@children` where the behavior is content-independent. A named, fixed example is gallery composition, even if its container is visually reusable.
- A public `width` or `height` default is a baseline, not a reason to position a smaller child implicitly. Inside a non-layout parent, set `x` and `y` when either child dimension differs from the parent. Let `HorizontalLayout`, `VerticalLayout`, or `FlexboxLayout` own flow and stretching where possible.
- Reserve border width across normal, hover, focus, selected, pressed, and disabled states so state changes do not shift nearby content. The same states must remain visible in dark and light themes.
- Give icon-only controls accessible names and ensure keyboard activation, Escape dismissal, and focus return follow the control's role.

## Dependency direction

```text
gallery and host
    ↓
omarchy.slint public exports → reusable styled modules
    ↓                         ↓
semantic Palette          Slint runtime and standard widgets
```

`gallery_examples.slint` may import reusable controls. Reusable controls must not import gallery examples or gallery state. The facade exports names; it does not contain implementation logic.

## Layout and regression checks

The gallery's side and content scrollers are 10 px narrower than their containing rectangles to reserve a scrollbar column. Both must be explicitly anchored at `x: 0px`: without an anchor, Slint centers each narrower child and introduces a 5 px alignment error. The same rule applies to every internally inset viewport. A component's visible border, content spine, and scrollbar must be checked together at the default and minimum window widths.

`Card` gives its children an inset. A nested layout passed through `@children` can still resolve `parent.width` against the outer `Card`, which makes that layout too wide and clips trailing icons. Bind the nested layout to `Card.content-width`, then bind its child rows to that layout's width. The Accordion gallery example exercises this case.

`cargo check --all-targets` validates the public facade through `examples/reuse.rs`. `cargo test` validates host logic. Use the software-rendered snapshot CLI to inspect interaction states and geometry; compile success alone does not prove visual alignment. The current Slint 1.18.1 idle comparison and its limits are recorded in [`performance.md`](performance.md).

The next architectural work is to audit remaining low-level helper exports and make common density and typography scale respond to a shared theme setting. Those changes should preserve the public facade's controlled-state contract.
