use slint::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    pub fn parse_hex(input: &str) -> Option<Self> {
        let digits = input.trim().strip_prefix('#').unwrap_or(input.trim());
        if !matches!(digits.len(), 3 | 4 | 6 | 8)
            || !digits.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return None;
        }
        let value = match digits.len() {
            3 | 4 => {
                let mut parts = digits.bytes().map(|byte| {
                    let digit = (byte as char).to_digit(16).unwrap() as u8;
                    digit * 17
                });
                Self {
                    r: parts.next()?,
                    g: parts.next()?,
                    b: parts.next()?,
                    a: parts.next().unwrap_or(255),
                }
            }
            6 | 8 => Self {
                r: u8::from_str_radix(&digits[0..2], 16).ok()?,
                g: u8::from_str_radix(&digits[2..4], 16).ok()?,
                b: u8::from_str_radix(&digits[4..6], 16).ok()?,
                a: if digits.len() == 8 {
                    u8::from_str_radix(&digits[6..8], 16).ok()?
                } else {
                    255
                },
            },
            _ => unreachable!(),
        };
        Some(value)
    }

    pub fn hex(self) -> String {
        if self.a == 255 {
            format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
        } else {
            format!("#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
        }
    }

    pub fn color(self) -> Color {
        Color::from_argb_u8(self.a, self.r, self.g, self.b)
    }

    pub fn hsla(self) -> [i32; 4] {
        let r = f64::from(self.r) / 255.0;
        let g = f64::from(self.g) / 255.0;
        let b = f64::from(self.b) / 255.0;
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;
        let light = (max + min) / 2.0;
        let sat = if delta == 0.0 {
            0.0
        } else {
            delta / (1.0 - (2.0 * light - 1.0).abs())
        };
        let hue = if delta == 0.0 {
            0.0
        } else if max == r {
            ((g - b) / delta).rem_euclid(6.0) / 6.0
        } else if max == g {
            ((b - r) / delta + 2.0) / 6.0
        } else {
            ((r - g) / delta + 4.0) / 6.0
        };
        [
            (hue * 100.0).round() as i32,
            (sat * 100.0).round() as i32,
            (light * 100.0).round() as i32,
            (f64::from(self.a) * 100.0 / 255.0).round() as i32,
        ]
    }

    pub fn from_hsla(hue: i32, sat: i32, light: i32, alpha: i32) -> Self {
        let hue = f64::from(hue.clamp(0, 100)) / 100.0;
        let sat = f64::from(sat.clamp(0, 100)) / 100.0;
        let light = f64::from(light.clamp(0, 100)) / 100.0;
        let chroma = (1.0 - (2.0 * light - 1.0).abs()) * sat;
        let sector = hue * 6.0;
        let x = chroma * (1.0 - (sector.rem_euclid(2.0) - 1.0).abs());
        let (r, g, b) = match sector as i32 {
            0 => (chroma, x, 0.0),
            1 => (x, chroma, 0.0),
            2 => (0.0, chroma, x),
            3 => (0.0, x, chroma),
            4 => (x, 0.0, chroma),
            _ => (chroma, 0.0, x),
        };
        let offset = light - chroma / 2.0;
        Self {
            r: ((r + offset) * 255.0).round() as u8,
            g: ((g + offset) * 255.0).round() as u8,
            b: ((b + offset) * 255.0).round() as u8,
            a: (f64::from(alpha.clamp(0, 100)) * 255.0 / 100.0).round() as u8,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Rgba;

    #[test]
    fn parses_supported_hex_lengths() {
        assert_eq!(Rgba::parse_hex("#F00").unwrap().hex(), "#FF0000");
        assert_eq!(Rgba::parse_hex("#F008").unwrap().hex(), "#FF000088");
        assert_eq!(Rgba::parse_hex("7aa2f7").unwrap().hex(), "#7AA2F7");
        assert_eq!(Rgba::parse_hex("#11223344").unwrap().hex(), "#11223344");
        assert!(Rgba::parse_hex("#12345").is_none());
        assert!(Rgba::parse_hex("#GG0000").is_none());
    }

    #[test]
    fn hsla_round_trip_is_close_for_palette_colors() {
        for hex in ["#7AA2F7", "#F7768E", "#9ECE6A", "#000000", "#FFFFFF"] {
            let original = Rgba::parse_hex(hex).unwrap();
            let [h, s, l, a] = original.hsla();
            let result = Rgba::from_hsla(h, s, l, a);
            assert!(original.r.abs_diff(result.r) <= 4, "{hex}");
            assert!(original.g.abs_diff(result.g) <= 4, "{hex}");
            assert!(original.b.abs_diff(result.b) <= 4, "{hex}");
        }
    }
}
