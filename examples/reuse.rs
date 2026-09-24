slint::slint! {
    import { Palette, OmButton, OmCheckBox, OmInput, OmSlider, OmTabs, OmProgress, OmAlert } from "../ui/omarchy.slint";

    export component ReuseDemo inherits Window {
        title: "Reusable Slint Omarchy controls";
        width: 420px;
        height: 360px;
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
        }
    }
}

fn main() -> Result<(), slint::PlatformError> {
    ReuseDemo::new()?.run()
}
