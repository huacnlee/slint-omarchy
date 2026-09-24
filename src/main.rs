use chrono::Datelike;
use slint::winit_030::WinitWindowAccessor;
use slint::{Color, ComponentHandle, Model, ModelRc, VecModel};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::{Duration, Instant};

mod calendar;
mod color;
mod snapshot;
mod theme;
mod toast;
mod tree;
use calendar::MonthPage;
use color::Rgba;
use theme::Theme;
use toast::SavedToastTimer;
use tree::TreeState;

slint::include_modules!();

const GROUPS: &[(&str, &[&str])] = &[
    ("Explore", &["overview"]),
    (
        "Actions",
        &["button", "button_group", "link", "toggle", "toggle_group"],
    ),
    (
        "Forms",
        &[
            "input",
            "textarea",
            "number_input",
            "select",
            "combobox",
            "calendar",
            "date_picker",
            "color_picker",
            "slider",
            "checkbox",
            "switch",
            "radio",
        ],
    ),
    (
        "Navigation",
        &[
            "menu",
            "tabs",
            "accordion",
            "collapsible",
            "nav_stack",
            "pagination",
        ],
    ),
    (
        "Overlays",
        &[
            "sheet",
            "dialog",
            "alert_dialog",
            "popover",
            "tooltip",
            "hover_card",
            "toast",
        ],
    ),
    (
        "Display",
        &[
            "icon",
            "avatar",
            "text_view",
            "panel",
            "virtual_list",
            "scrollbar",
            "table",
            "tree",
            "resizable",
            "separator",
            "keycap",
            "badge",
            "alert",
            "empty_state",
            "progress",
        ],
    ),
];

const IMPLEMENTED: &[&str] = &[
    "overview",
    "button",
    "button_group",
    "toggle",
    "input",
    "number_input",
    "slider",
    "checkbox",
    "switch",
    "tabs",
    "accordion",
    "collapsible",
    "dialog",
    "alert_dialog",
    "toast",
    "empty_state",
    "progress",
    "virtual_list",
    "scrollbar",
    "textarea",
    "select",
    "combobox",
    "table",
    "badge",
    "alert",
    "separator",
    "keycap",
    "avatar",
    "panel",
    "calendar",
    "date_picker",
    "toggle_group",
    "radio",
    "pagination",
    "color_picker",
    "link",
    "icon",
    "menu",
    "nav_stack",
    "popover",
    "tooltip",
    "hover_card",
    "sheet",
    "tree",
    "text_view",
    "resizable",
];

fn display_name(page: &str) -> String {
    let label = page.replace('_', " ");
    let mut chars = label.chars();
    chars.next().map_or(String::new(), |first| {
        first.to_uppercase().collect::<String>() + chars.as_str()
    })
}

fn next_enabled_choice(items: &ModelRc<ChoiceItem>, current: i32, direction: i32) -> i32 {
    let enabled = (0..items.row_count())
        .filter(|&index| items.row_data(index).is_some_and(|item| item.enabled))
        .map(|index| index as i32)
        .collect::<Vec<_>>();
    if enabled.is_empty() {
        return -1;
    }
    if direction >= 100 {
        return enabled[0];
    }
    if direction <= -100 {
        return *enabled.last().unwrap();
    }
    let position = enabled.iter().position(|&index| index == current);
    if direction < 0 {
        enabled[position.map_or(enabled.len() - 1, |index| {
            (index + enabled.len() - 1) % enabled.len()
        })]
    } else {
        enabled[position.map_or(0, |index| (index + 1) % enabled.len())]
    }
}

fn page_description(page: &str) -> &'static str {
    match page {
        "overview" => "Explore the components together, then use the sidebar to inspect each one.",
        "button" => "Content-sized actions with quiet hover, focus and pressed states.",
        "button_group" => "Choose one setting from a row of mutually exclusive options.",
        "link" => "Open a named destination.",
        "toggle" => "Keep a command active until it is pressed again.",
        "toggle_group" => "Combine independent filters to show more than one status.",
        "input" => "Single-line editing with selection, clipboard and IME support.",
        "textarea" => "Multi-line notes with native text editing.",
        "number_input" => "A compact numeric field with keyboard and button stepping.",
        "select" => "Choose one value from a fixed set of options.",
        "combobox" => "Search a collection, then choose a matching option.",
        "calendar" => "Choose a date with month and year navigation.",
        "date_picker" => "Choose a date from a calendar anchored to a field.",
        "color_picker" => "Choose a label color with Hex and HSLA controls.",
        "slider" => "Adjust one value or a range with the pointer or keyboard.",
        "checkbox" => "Independent choices, including a mixed selection.",
        "switch" => "An immediate on/off choice with a visible track and thumb.",
        "radio" => "Choose one option from a group.",
        "menu" => "An anchored action menu with one pointer and keyboard cursor.",
        "tabs" => "Switch between related views while keeping context.",
        "accordion" => "Reveal supporting content when it is needed.",
        "collapsible" => "Expand a single region for additional settings.",
        "nav_stack" => "Navigate between persistent pages and return to where you left off.",
        "pagination" => "Move through a paged collection.",
        "sheet" => "Inspect project details in an edge-attached panel.",
        "dialog" => "A focused task with confirmation, cancellation and focus return.",
        "alert_dialog" => "An explicit decision that cannot be dismissed by clicking the backdrop.",
        "popover" => "Adjust contextual settings while keeping the workspace in view.",
        "tooltip" => "A short explanation for an action, shown on hover.",
        "hover_card" => "Preview supporting details without leaving the page.",
        "toast" => "Brief feedback that leaves your current task in place.",
        "icon" => "Monochrome SVG icons that inherit the surrounding text color.",
        "avatar" => "Identify people and workspaces with a square image or initials.",
        "text_view" => "Read structured documents with selectable text and links.",
        "panel" => "A surface for a related group of settings or information.",
        "virtual_list" => "Browse a large activity log with variable-height rows.",
        "scrollbar" => "Drag the scroll thumb to move through a long activity log.",
        "table" => "Aligned columns for comparing records.",
        "tree" => "Explore nested folders and select a workspace document.",
        "resizable" => "Drag the divider to adjust space between panes.",
        "separator" => "A quiet boundary between distinct sections.",
        "keycap" => "Compact, readable keyboard hints.",
        "badge" => "Short labels for neutral and semantic status.",
        "alert" => "Inline feedback banners for neutral and semantic conditions.",
        "empty_state" => "Explain an empty collection and its next step.",
        "progress" => "Show how much of a known task is complete.",
        _ => "",
    }
}

fn main() -> Result<(), slint::PlatformError> {
    let snapshot_path = std::env::args().skip(1).find_map(|arg| {
        arg.strip_prefix("--snapshot=")
            .map(std::path::PathBuf::from)
    });
    let snapshot_size = std::env::args()
        .skip(1)
        .find_map(|arg| {
            let (width, height) = arg.strip_prefix("--snapshot-size=")?.split_once('x')?;
            let width = width.parse::<u32>().ok()?;
            let height = height.parse::<u32>().ok()?;
            (width > 0 && height > 0).then_some((width, height))
        })
        .unwrap_or((1060, 760));
    let snapshot_clicks = std::env::args()
        .skip(1)
        .filter_map(|arg| {
            let (x, y) = arg.strip_prefix("--click=")?.split_once(',')?;
            Some((x.parse::<f32>().ok()?, y.parse::<f32>().ok()?))
        })
        .collect::<Vec<_>>();
    let snapshot_hovers = std::env::args()
        .skip(1)
        .filter_map(|arg| {
            let (x, y) = arg.strip_prefix("--hover=")?.split_once(',')?;
            Some((x.parse::<f32>().ok()?, y.parse::<f32>().ok()?))
        })
        .collect::<Vec<_>>();
    let snapshot_drags = std::env::args()
        .skip(1)
        .filter_map(|arg| {
            let coordinates = arg
                .strip_prefix("--drag=")?
                .split(',')
                .map(str::parse::<f32>)
                .collect::<Result<Vec<_>, _>>()
                .ok()?;
            (coordinates.len() == 4).then(|| {
                (
                    coordinates[0],
                    coordinates[1],
                    coordinates[2],
                    coordinates[3],
                )
            })
        })
        .collect::<Vec<_>>();
    let snapshot_text = std::env::args()
        .skip(1)
        .find_map(|arg| arg.strip_prefix("--type=").map(str::to_owned));
    let snapshot_keys = std::env::args()
        .skip(1)
        .filter_map(|arg| arg.strip_prefix("--key=").map(str::to_owned))
        .collect::<Vec<_>>();
    let snapshot_wait_ms = std::env::args()
        .skip(1)
        .find_map(|arg| {
            arg.strip_prefix("--wait-ms=")
                .and_then(|value| value.parse::<u64>().ok())
        })
        .unwrap_or(0);
    if snapshot_path.is_some() {
        snapshot::install(snapshot_size);
    } else {
        slint::BackendSelector::new()
            .backend_name("winit".into())
            .with_winit_window_attributes_hook(|attributes| {
                attributes.with_min_inner_size(slint::winit_030::winit::dpi::LogicalSize::new(
                    680.0, 520.0,
                ))
            })
            .select()?;
    }
    let app = Gallery::new()?;
    if snapshot_path.is_none() {
        app.window()
            .set_size(slint::LogicalSize::new(1060.0, 760.0));
    }
    let base_scale_factor = app.window().scale_factor();
    let weak = app.as_weak();
    app.on_set_zoom(move |percent| {
        if let Some(app) = weak.upgrade() {
            let percent = percent.clamp(50, 200);
            let physical_size = app.window().size();
            let scale_factor = base_scale_factor * percent as f32 / 100.0;
            app.set_zoom_percent(percent);
            app.window()
                .dispatch_event(slint::platform::WindowEvent::ScaleFactorChanged { scale_factor });
            app.window().set_size(physical_size);
            app.window()
                .dispatch_event(slint::platform::WindowEvent::Resized {
                    size: slint::LogicalSize::new(
                        physical_size.width as f32 / scale_factor,
                        physical_size.height as f32 / scale_factor,
                    ),
                });
        }
    });
    let weak = app.as_weak();
    app.on_drag_window(move || {
        if let Some(app) = weak.upgrade() {
            app.window().with_winit_window(|window| {
                let _ = window.drag_window();
            });
        }
    });
    let weak = app.as_weak();
    app.on_minimize_window(move || {
        if let Some(app) = weak.upgrade() {
            app.window()
                .with_winit_window(|window| window.set_minimized(true));
        }
    });
    let weak = app.as_weak();
    app.on_maximize_window(move || {
        if let Some(app) = weak.upgrade() {
            app.window()
                .with_winit_window(|window| window.set_maximized(!window.is_maximized()));
        }
    });
    app.on_close_window(|| {
        let _ = slint::quit_event_loop();
    });
    let saved_toast_timer = Rc::new(RefCell::new(SavedToastTimer::default()));
    let weak = app.as_weak();
    let timer_state = saved_toast_timer.clone();
    app.on_restart_toast_timer(move || {
        if let Some(app) = weak.upgrade() {
            timer_state
                .borrow_mut()
                .restart(Instant::now(), app.get_toast_paused());
        }
    });
    let timer_state = saved_toast_timer.clone();
    app.on_toast_pause_changed(move |paused| {
        timer_state.borrow_mut().set_paused(Instant::now(), paused);
    });
    let timer_state = saved_toast_timer.clone();
    app.on_dismiss_toast_timer(move || timer_state.borrow_mut().dismiss());
    let toast_timer = slint::Timer::default();
    let weak = app.as_weak();
    toast_timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(100),
        move || {
            if let Some(app) = weak.upgrade() {
                let expired =
                    app.get_toast_open() && saved_toast_timer.borrow().expired(Instant::now());
                if expired {
                    saved_toast_timer.borrow_mut().dismiss();
                    app.set_toast_open(false);
                }
            }
        },
    );
    let weak = app.as_weak();
    app.on_submit_input(move |value| {
        if let Some(app) = weak.upgrade() {
            app.set_input_submitted_length(value.chars().count() as i32);
        }
    });
    app.on_check_workspace_name(|value| !value.trim().is_empty());
    let weak = app.as_weak();
    app.on_save_workspace_name(move |value| {
        if let Some(app) = weak.upgrade() {
            let name = value.trim();
            if name.is_empty() {
                app.set_dialog_result("Enter a workspace name".into());
            } else {
                app.set_saved_workspace(name.into());
                app.set_inline_workspace_name(name.into());
                app.set_dialog_workspace_name(name.into());
                app.set_inline_workspace_valid(true);
                app.set_dialog_workspace_valid(true);
                app.set_dialog_result(format!("Saved “{name}”").into());
            }
        }
    });
    let weak = app.as_weak();
    app.on_reset_workspace(move || {
        if let Some(app) = weak.upgrade() {
            app.set_saved_workspace("Personal workspace".into());
            app.set_inline_workspace_name("Personal workspace".into());
            app.set_dialog_workspace_name("Personal workspace".into());
            app.set_inline_workspace_valid(true);
            app.set_dialog_workspace_valid(true);
            app.set_dialog_result("Workspace defaults restored".into());
        }
    });
    let pages = GROUPS
        .iter()
        .flat_map(|(category, names)| {
            names.iter().map(move |name| NavItem {
                name: (*name).into(),
                label: display_name(name).into(),
                description: page_description(name).into(),
                category: (*category).into(),
                implemented: IMPLEMENTED.contains(name),
            })
        })
        .collect::<Vec<_>>();
    app.set_pages(ModelRc::new(VecModel::from(pages)));
    let activities = (0..1_000)
        .map(|index| ActivityItem {
            title: format!("Event {:04} · Updated project notes", index + 1).into(),
            detail: "Review requested by Alex Lee".into(),
            tall: index % 5 == 0,
        })
        .collect::<Vec<_>>();
    app.set_activities(ModelRc::new(VecModel::from(activities)));
    let choices = [
        ("personal", "Personal", true),
        ("studio", "Studio", true),
        ("research", "Research", true),
        ("archive", "Archive", false),
    ]
    .into_iter()
    .map(|(value, label, enabled)| ChoiceItem {
        value: value.into(),
        label: label.into(),
        enabled,
    })
    .collect::<Vec<_>>();
    app.set_workspace_choices(ModelRc::new(VecModel::from(choices.clone())));
    app.set_filtered_choices(ModelRc::new(VecModel::from(choices.clone())));
    let weak = app.as_weak();
    app.on_next_choice_index(move |current, direction, searchable| {
        weak.upgrade().map_or(-1, |app| {
            let items = if searchable {
                app.get_filtered_choices()
            } else {
                app.get_workspace_choices()
            };
            next_enabled_choice(&items, current, direction)
        })
    });
    let weak = app.as_weak();
    app.on_filter_choices(move |query| {
        if let Some(app) = weak.upgrade() {
            let query = query.to_lowercase();
            let filtered = choices
                .iter()
                .filter(|item| item.label.to_lowercase().contains(&query))
                .cloned()
                .collect::<Vec<_>>();
            app.set_filtered_choices(ModelRc::new(VecModel::from(filtered)));
        }
    });
    let projects = [
        ("Website refresh", "Active", "Alex Lee", "Sep 7", "12 files"),
        ("Design system", "Active", "Morgan Kim", "Sep 6", "28 files"),
        ("Release notes", "Review", "Sam Rivera", "Sep 5", "4 files"),
        ("Desktop client", "Active", "Alex Lee", "Sep 4", "36 files"),
        ("Onboarding", "Review", "Morgan Kim", "Sep 3", "9 files"),
        (
            "Research archive",
            "Archived",
            "Sam Rivera",
            "Aug 28",
            "42 files",
        ),
        ("API reference", "Active", "Alex Lee", "Aug 26", "17 files"),
        (
            "Brand assets",
            "Archived",
            "Morgan Kim",
            "Aug 21",
            "24 files",
        ),
    ]
    .into_iter()
    .map(|(name, status, owner, updated, files)| ProjectRow {
        name: name.into(),
        status: status.into(),
        owner: owner.into(),
        updated: updated.into(),
        files: files.into(),
    })
    .collect::<Vec<_>>();
    app.set_projects(ModelRc::new(VecModel::from(projects)));
    let tree = Rc::new(RefCell::new(TreeState::default()));
    refresh_tree(&app, &tree.borrow());
    let weak = app.as_weak();
    let state = tree.clone();
    app.on_tree_activate(move |index| {
        if let Some(app) = weak.upgrade() {
            state.borrow_mut().activate(index as usize);
            refresh_tree(&app, &state.borrow());
        }
    });
    let weak = app.as_weak();
    let state = tree.clone();
    app.on_tree_move(move |delta| {
        if let Some(app) = weak.upgrade() {
            state.borrow_mut().move_cursor(delta);
            refresh_tree(&app, &state.borrow());
        }
    });
    let weak = app.as_weak();
    let state = tree.clone();
    app.on_tree_left(move || {
        if let Some(app) = weak.upgrade() {
            state.borrow_mut().left();
            refresh_tree(&app, &state.borrow());
        }
    });
    let weak = app.as_weak();
    let state = tree.clone();
    app.on_tree_right(move || {
        if let Some(app) = weak.upgrade() {
            state.borrow_mut().right();
            refresh_tree(&app, &state.borrow());
        }
    });
    let today = chrono::Local::now().date_naive();
    apply_month(
        &app,
        MonthPage::new(today.year(), today.month()).expect("current month"),
    );
    let weak = app.as_weak();
    app.on_shift_month(move |delta| {
        if let Some(app) = weak.upgrade() {
            if let Some(month) =
                MonthPage::new(app.get_calendar_year(), app.get_calendar_month() as u32)
                    .and_then(|page| page.shifted(delta))
            {
                apply_month(&app, month);
            }
        }
    });
    let weak = app.as_weak();
    app.on_jump_calendar_month(move |month| {
        if let Some(app) = weak.upgrade() {
            if let Some(page) = MonthPage::new(app.get_calendar_year(), month as u32) {
                apply_month(&app, page);
            }
        }
    });
    let weak = app.as_weak();
    app.on_jump_calendar_year(move |year| {
        if let Some(app) = weak.upgrade() {
            if let Some(page) = MonthPage::new(year, app.get_calendar_month() as u32) {
                apply_month(&app, page);
            }
        }
    });
    let weak = app.as_weak();
    app.on_select_day(move |day| {
        if let Some(app) = weak.upgrade() {
            if let Some(date) =
                MonthPage::new(app.get_calendar_year(), app.get_calendar_month() as u32)
                    .and_then(|page| page.selectable_date(day as u32))
            {
                apply_selected_date(&app, date);
            }
        }
    });
    let weak = app.as_weak();
    app.on_select_adjacent_day(move |offset, day| {
        if let Some(app) = weak.upgrade() {
            if let Some(date) =
                MonthPage::new(app.get_calendar_year(), app.get_calendar_month() as u32)
                    .and_then(|page| page.selectable_offset_date(offset, day as u32))
            {
                if let Some(page) = MonthPage::new(date.year(), date.month()) {
                    apply_month(&app, page);
                }
                apply_selected_date(&app, date);
            }
        }
    });
    if let Some(page) = std::env::args()
        .skip(1)
        .find_map(|arg| arg.strip_prefix("--page=").map(str::to_owned))
    {
        if GROUPS
            .iter()
            .any(|(_, names)| names.contains(&page.as_str()))
        {
            app.set_current_page(page.clone().into());
            app.set_current_page_label(display_name(&page).into());
            app.set_current_page_description(page_description(&page).into());
            app.set_current_implemented(IMPLEMENTED.contains(&page.as_str()));
        }
    }
    let initial_theme = Theme::system_or_default();
    let initial_color = Rgba {
        r: initial_theme.accent[0],
        g: initial_theme.accent[1],
        b: initial_theme.accent[2],
        a: 255,
    };
    commit_color(&app, initial_color);
    apply_theme(&app, initial_theme.clone());
    let following_system_theme = Rc::new(Cell::new(true));
    let last_system_theme = Rc::new(RefCell::new(initial_theme));
    let weak = app.as_weak();
    app.on_open_color(move || {
        if let Some(app) = weak.upgrade() {
            restore_color(&app);
            app.set_color_open(true);
        }
    });
    let weak = app.as_weak();
    app.on_preview_color_hex(move |input| {
        if let Some(app) = weak.upgrade() {
            match Rgba::parse_hex(&input) {
                Some(color) => {
                    preview_color(&app, color);
                    app.set_color_invalid(false);
                }
                None => app.set_color_invalid(true),
            }
        }
    });
    let weak = app.as_weak();
    app.on_change_color_channel(move |hue, sat, light, alpha| {
        if let Some(app) = weak.upgrade() {
            let color = Rgba::from_hsla(hue, sat, light, alpha);
            app.set_color_preview(color.color());
            app.set_color_draft_hex(color.hex().into());
            app.set_color_invalid(false);
        }
    });
    let weak = app.as_weak();
    app.on_commit_color(move |input| {
        if let Some(app) = weak.upgrade() {
            if let Some(color) = Rgba::parse_hex(&input) {
                commit_color(&app, color);
                app.set_color_open(false);
                app.set_color_invalid(false);
            } else {
                app.set_color_invalid(true);
            }
        }
    });
    let weak = app.as_weak();
    app.on_cancel_color(move || {
        if let Some(app) = weak.upgrade() {
            restore_color(&app);
            app.set_color_open(false);
        }
    });
    let weak = app.as_weak();
    app.on_open_manual(move || {
        if let Some(app) = weak.upgrade() {
            let message = match webbrowser::open("https://omarchy.org/manual") {
                Ok(()) => "Opened Omarchy manual",
                Err(_) => "Could not open Omarchy manual",
            };
            app.set_status(message.into());
        }
    });
    let weak = app.as_weak();
    app.on_open_source(move || {
        if let Some(app) = weak.upgrade() {
            let message = match webbrowser::open("https://github.com/huacnlee/gpui-omarchy") {
                Ok(()) => "Opened project source",
                Err(_) => "Could not open project source",
            };
            app.set_status(message.into());
        }
    });
    let weak = app.as_weak();
    let system_mode = following_system_theme.clone();
    let previous_theme = last_system_theme.clone();
    app.on_reload_theme(move || {
        if let Some(app) = weak.upgrade() {
            let theme = Theme::system_or_default();
            system_mode.set(true);
            *previous_theme.borrow_mut() = theme.clone();
            apply_theme(&app, theme);
        }
    });
    let weak = app.as_weak();
    let system_mode = following_system_theme.clone();
    app.on_use_dark_theme(move || {
        if let Some(app) = weak.upgrade() {
            system_mode.set(false);
            apply_theme(&app, Theme::tokyo_night());
        }
    });
    let weak = app.as_weak();
    let system_mode = following_system_theme.clone();
    app.on_use_light_theme(move || {
        if let Some(app) = weak.upgrade() {
            system_mode.set(false);
            apply_theme(&app, Theme::flexoki_light());
        }
    });
    let theme_timer = slint::Timer::default();
    let weak = app.as_weak();
    theme_timer.start(
        slint::TimerMode::Repeated,
        Duration::from_secs(2),
        move || {
            if !following_system_theme.get() {
                return;
            }
            let Some(app) = weak.upgrade() else {
                return;
            };
            let theme = Theme::system_or_default();
            if *last_system_theme.borrow() != theme {
                *last_system_theme.borrow_mut() = theme.clone();
                apply_theme(&app, theme);
            }
        },
    );
    let weak = app.as_weak();
    app.on_move_selection(move |delta| {
        if let Some(app) = weak.upgrade() {
            let pages = GROUPS
                .iter()
                .flat_map(|(_, names)| names.iter().copied())
                .collect::<Vec<_>>();
            let current = pages
                .iter()
                .position(|name| app.get_current_page() == *name)
                .unwrap_or(0);
            let next = if delta < -1 {
                0
            } else if delta > 1 {
                pages.len() - 1
            } else {
                (current as isize + delta as isize).rem_euclid(pages.len() as isize) as usize
            };
            let name = pages[next];
            app.set_current_page(name.into());
            app.set_current_page_label(display_name(name).into());
            app.set_current_page_description(page_description(name).into());
            app.set_current_implemented(IMPLEMENTED.contains(&name));
            app.set_status(format!("Viewing {}", display_name(name)).into());

            // Keep keyboard selection inside the sidebar viewport. The first
            // group header is 30px; later groups add 38px. Rows are 32px with
            // a 2px gap, matching the gallery's navigation layout.
            let mut first_in_group = 0;
            let mut group_index = 0;
            for (index, (_, names)) in GROUPS.iter().enumerate() {
                if next < first_in_group + names.len() {
                    group_index = index;
                    break;
                }
                first_in_group += names.len();
            }
            let row_top = 30.0 + 34.0 * next as f32 + 38.0 * group_index as f32;
            let row_bottom = row_top + 32.0;
            let visible_height = app.get_nav_visible_height();
            if visible_height > 0.0 {
                let viewport_top = -app.get_nav_scroll_y();
                let viewport_bottom = viewport_top + visible_height;
                let next_top = if next == 0 {
                    0.0
                } else if row_top < viewport_top {
                    row_top
                } else if row_bottom > viewport_bottom {
                    row_bottom - visible_height
                } else {
                    viewport_top
                };
                app.set_nav_scroll_y(-next_top.max(0.0));
            }
        }
    });
    if let Some(path) = snapshot_path {
        snapshot::save(
            &app,
            &path,
            &snapshot_clicks,
            &snapshot_hovers,
            &snapshot_drags,
            snapshot_text.as_deref(),
            &snapshot_keys,
            snapshot_wait_ms,
            snapshot_size,
        );
        return Ok(());
    }
    app.run()
}

fn apply_theme(app: &Gallery, theme: Theme) {
    let palette = app.global::<Palette>();
    let color = |rgb: [u8; 3]| Color::from_rgb_u8(rgb[0], rgb[1], rgb[2]);
    palette.set_name(theme.name.into());
    palette.set_background(color(theme.background));
    palette.set_surface(color(theme.surface));
    palette.set_inset(color(theme.inset));
    palette.set_foreground(color(theme.foreground));
    palette.set_bright(color(theme.bright));
    palette.set_accent(color(theme.accent));
    palette.set_border(color(theme.border));
    palette.set_muted(color(theme.muted));
    palette.set_danger(color(theme.danger));
    palette.set_warning(color(theme.warning));
    palette.set_success(color(theme.success));
}

fn apply_month(app: &Gallery, page: MonthPage) {
    if let Some(previous) = page.shifted(-1) {
        app.set_calendar_previous_month_label(
            format!("{} {}", previous.label, previous.year).into(),
        );
    }
    if let Some(next) = page.shifted(1) {
        app.set_calendar_next_month_label(format!("{} {}", next.label, next.year).into());
    }
    app.set_calendar_year(page.year);
    app.set_calendar_month(page.month as i32);
    app.set_calendar_month_label(page.label.into());
    app.set_calendar_leading(page.leading as i32);
    app.set_calendar_days(page.days as i32);
    app.set_calendar_previous_days(page.previous_days as i32);
}

fn apply_selected_date(app: &Gallery, date: chrono::NaiveDate) {
    app.set_selected_year(date.year());
    app.set_selected_month(date.month() as i32);
    app.set_selected_day(date.day() as i32);
    app.set_selected_date_label(date.format("%A, %B %-d, %Y").to_string().into());
    app.set_selected_short_date_label(date.format("%b %-d, %Y").to_string().into());
}

fn preview_color(app: &Gallery, color: Rgba) {
    app.set_color_preview(color.color());
    let [hue, saturation, lightness, alpha] = color.hsla();
    app.set_color_hue(hue);
    app.set_color_saturation(saturation);
    app.set_color_lightness(lightness);
    app.set_color_alpha(alpha);
}

fn commit_color(app: &Gallery, color: Rgba) {
    preview_color(app, color);
    app.set_color_committed(color.color());
    app.set_color_committed_hex(color.hex().into());
    app.set_color_draft_hex(color.hex().into());
}

fn restore_color(app: &Gallery) {
    if let Some(color) = Rgba::parse_hex(&app.get_color_committed_hex()) {
        preview_color(app, color);
        app.set_color_draft_hex(color.hex().into());
        app.set_color_invalid(false);
    }
}

fn refresh_tree(app: &Gallery, state: &TreeState) {
    let rows = state
        .visible()
        .into_iter()
        .map(|row| TreeRow {
            id: row.id.into(),
            label: row.label.into(),
            depth: row.depth,
            folder: row.folder,
            expanded: row.expanded,
            disabled: row.disabled,
        })
        .collect::<Vec<_>>();
    app.set_tree_rows(ModelRc::new(VecModel::from(rows)));
    app.set_tree_cursor(state.cursor_index() as i32);
    app.set_tree_selected_id(state.selected_id().unwrap_or_default().into());
    app.set_tree_selected_label(state.selected_label().unwrap_or_default().into());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn choice_navigation_skips_disabled_options_and_wraps() {
        let items = [true, false, false, true]
            .into_iter()
            .map(|enabled| ChoiceItem {
                value: "value".into(),
                label: "Choice".into(),
                enabled,
            })
            .collect::<Vec<_>>();
        let model = ModelRc::new(VecModel::from(items));
        assert_eq!(next_enabled_choice(&model, -1, 100), 0);
        assert_eq!(next_enabled_choice(&model, 0, 1), 3);
        assert_eq!(next_enabled_choice(&model, 3, 1), 0);
        assert_eq!(next_enabled_choice(&model, 0, -1), 3);
        assert_eq!(next_enabled_choice(&model, -1, -100), 3);
        let empty = ModelRc::new(VecModel::from(vec![ChoiceItem {
            value: "disabled".into(),
            label: "Disabled".into(),
            enabled: false,
        }]));
        assert_eq!(next_enabled_choice(&empty, -1, 100), -1);
    }
}
