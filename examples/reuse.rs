slint::slint! {
    import { Palette, Label, OmButton, OmCheckBox, OmInput, OmSlider, OmTabs, OmProgress, OmAlert, OmTable, OmTableRow, OmTableCell } from "../ui/omarchy.slint";

    export component ReuseDemo inherits Window {
        title: "Reusable Slint Omarchy controls";
        width: 420px;
        height: 460px;
        background: Palette.background;

        in-out property <bool> checked: false;
        in-out property <int> volume: 40;
        in-out property <string> workspace: "";
        in-out property <int> active-tab: 0;

        VerticalLayout {
            padding: 20px;
            spacing: 12px;
            OmButton { label: "Apply"; primary: true; clicked => { root.checked = true; } }
            OmInput { text <=> root.workspace; placeholder: "Workspace name"; accessible-name: "Workspace name"; }
            OmCheckBox { label: "Include hidden files"; checked <=> root.checked; }
            OmSlider { label: "Volume"; value <=> root.volume; }
            OmTabs { options: ["General", "Advanced"]; selected <=> root.active-tab; }
            OmProgress { value: root.volume; }
            OmAlert { neutral: true; message: "Components imported from ui/omarchy.slint"; }
            OmTable {
                width: 300px;
                accessible-name: "Services";
                OmTableRow {
                    header: true; row-index: 1; accessible-name: "Service, State";
                    OmTableCell { width: 140px; Label { text: "Service"; color: Palette.bright; font-weight: 600; } }
                    OmTableCell { width: 140px; Label { text: "State"; color: Palette.bright; font-weight: 600; } }
                }
                OmTableRow {
                    row-index: 2; accessible-name: "Indexer, Ready";
                    OmTableCell { width: 140px; Label { text: "Indexer"; } }
                    OmTableCell { width: 140px; Label { text: "Ready"; } }
                }
            }
        }
    }
}

fn main() -> Result<(), slint::PlatformError> {
    ReuseDemo::new()?.run()
}
