use std::{fs, path::Path};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Theme {
    pub name: String,
    pub background: [u8; 3],
    pub surface: [u8; 3],
    pub inset: [u8; 3],
    pub foreground: [u8; 3],
    pub bright: [u8; 3],
    pub accent: [u8; 3],
    pub border: [u8; 3],
    pub muted: [u8; 3],
    pub danger: [u8; 3],
    pub warning: [u8; 3],
    pub success: [u8; 3],
}

impl Theme {
    pub fn tokyo_night() -> Self {
        Self {
            name: "Tokyo Night".into(),
            background: hex("#1a1b26").unwrap(),
            surface: hex("#24283b").unwrap(),
            inset: hex("#13141c").unwrap(),
            foreground: hex("#a9b1d6").unwrap(),
            bright: hex("#c0caf5").unwrap(),
            accent: hex("#7aa2f7").unwrap(),
            border: hex("#414868").unwrap(),
            muted: hex("#737aa2").unwrap(),
            danger: hex("#f7768e").unwrap(),
            warning: hex("#e0af68").unwrap(),
            success: hex("#9ece6a").unwrap(),
        }
    }

    pub fn flexoki_light() -> Self {
        Self {
            name: "Flexoki Light".into(),
            background: hex("#fffcf0").unwrap(),
            surface: hex("#f2f0e5").unwrap(),
            inset: hex("#e6e4d9").unwrap(),
            foreground: hex("#100f0f").unwrap(),
            bright: hex("#100f0f").unwrap(),
            accent: hex("#205ea6").unwrap(),
            border: hex("#b7b5ac").unwrap(),
            muted: hex("#575653").unwrap(),
            danger: hex("#af3029").unwrap(),
            warning: hex("#855b00").unwrap(),
            success: hex("#526600").unwrap(),
        }
    }

    pub fn system_or_default() -> Self {
        let Some(home) = std::env::var_os("HOME") else {
            return Self::tokyo_night();
        };
        let home = Path::new(&home);
        let current = home.join(".local/state/omarchy/current");
        let path = match fs::symlink_metadata(&current) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                home.join(".config/omarchy/current")
            }
            _ => current,
        };
        Self::from_current_dir(&path).unwrap_or_else(|_| Self::tokyo_night())
    }

    pub fn from_current_dir(path: &Path) -> Result<Self, String> {
        let contents = fs::read_to_string(path.join("theme/colors.toml"))
            .map_err(|error| error.to_string())?;
        let name = fs::read_to_string(path.join("theme.name"))
            .ok()
            .filter(|name| !name.trim().is_empty())
            .unwrap_or_else(|| "Omarchy".into());
        Self::from_colors_toml(name.trim(), &contents)
    }

    pub fn from_colors_toml(name: &str, contents: &str) -> Result<Self, String> {
        let values: toml::Table = contents.parse::<toml::Table>().map_err(|e| e.to_string())?;
        let color = |keys: &[&str]| -> Result<Option<[u8; 3]>, String> {
            for key in keys {
                if let Some(value) = values.get(*key) {
                    let value = value.as_str().ok_or(format!("{key} must be a color"))?;
                    return hex(value)
                        .map(Some)
                        .ok_or(format!("{key} must use #RRGGBB"));
                }
            }
            Ok(None)
        };
        let required = |keys: &[&str]| -> Result<[u8; 3], String> {
            color(keys)?.ok_or(format!("missing {}", keys[0]))
        };
        let background = required(&["background"])?;
        let foreground = required(&["foreground"])?;
        let accent = required(&["accent"])?;
        let light = match values.get("mode") {
            Some(value) if value.as_str() == Some("light") => true,
            Some(value) if value.as_str() == Some("dark") => false,
            Some(_) => return Err("mode must be dark or light".into()),
            None => luminance(background) > luminance(foreground),
        };
        let surface =
            color(&["lighter_background"])?.unwrap_or_else(|| mix(background, foreground, 0.05));
        let inset =
            color(&["dark_background"])?.unwrap_or_else(|| mix(background, foreground, 0.08));
        let bright =
            color(&["bright_foreground", "selection_foreground", "cursor"])?.unwrap_or(foreground);
        let border =
            color(&["muted", "color8"])?.unwrap_or_else(|| mix(background, foreground, 0.25));
        let muted = mix(background, foreground, 0.75);
        let mut danger = required(&["red", "color1"])?;
        let mut warning = required(&["yellow", "color3"])?;
        let mut success = required(&["green", "color2"])?;
        if light {
            for status in [&mut danger, &mut warning, &mut success] {
                while contrast(*status, background) < 4.5 {
                    *status = mix(*status, [0, 0, 0], 0.1);
                }
            }
        }
        Ok(Self {
            name: name.into(),
            background,
            surface,
            inset,
            foreground,
            bright,
            accent,
            border,
            muted,
            danger,
            warning,
            success,
        })
    }
}

fn hex(value: &str) -> Option<[u8; 3]> {
    let raw = value.strip_prefix('#')?;
    if raw.len() != 6 || !raw.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    Some([
        u8::from_str_radix(&raw[0..2], 16).ok()?,
        u8::from_str_radix(&raw[2..4], 16).ok()?,
        u8::from_str_radix(&raw[4..6], 16).ok()?,
    ])
}

fn mix(a: [u8; 3], b: [u8; 3], amount: f64) -> [u8; 3] {
    std::array::from_fn(|i| (a[i] as f64 + (b[i] as f64 - a[i] as f64) * amount).round() as u8)
}

fn luminance(rgb: [u8; 3]) -> f64 {
    let channels = rgb.map(|value| {
        let value = value as f64 / 255.0;
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    });
    channels[0] * 0.2126 + channels[1] * 0.7152 + channels[2] * 0.0722
}

fn contrast(a: [u8; 3], b: [u8; 3]) -> f64 {
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_semantic_and_ansi_palettes() {
        for colors in [
            "background='#1a1b26'\nforeground='#a9b1d6'\naccent='#7aa2f7'\nred='#f7768e'\nyellow='#e0af68'\ngreen='#9ece6a'",
            "background='#1a1b26'\nforeground='#a9b1d6'\naccent='#7aa2f7'\ncolor1='#f7768e'\ncolor3='#e0af68'\ncolor2='#9ece6a'",
        ] {
            let theme = Theme::from_colors_toml("Test", colors).unwrap();
            assert_eq!(theme.background, [0x1a, 0x1b, 0x26]);
            assert_eq!(theme.accent, [0x7a, 0xa2, 0xf7]);
        }
    }

    #[test]
    fn rejects_incomplete_or_malformed_palette() {
        assert!(Theme::from_colors_toml("Test", "background='#123456'").is_err());
        assert!(Theme::from_colors_toml("Test", "background='red'").is_err());
    }
}
