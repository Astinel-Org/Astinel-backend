use std::io::IsTerminal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorPreference {
    Auto,
    Always,
    Never,
}

#[derive(Debug, Clone)]
pub struct TerminalCapabilities {
    pub color: bool,
    pub unicode: bool,
    pub width: usize,
}

impl TerminalCapabilities {
    pub fn detect(color_pref: ColorPreference) -> Self {
        let color = match color_pref {
            ColorPreference::Always => true,
            ColorPreference::Never => false,
            ColorPreference::Auto => {
                if std::env::var("NO_COLOR").is_ok()
                    || std::env::var("TERM").is_ok_and(|v| v == "dumb")
                {
                    false
                } else {
                    std::io::stdout().is_terminal()
                }
            }
        };

        let unicode =
            std::io::stdout().is_terminal() && std::env::var("TERM").map_or(true, |v| v != "dumb");

        let width = terminal_width().unwrap_or(80);

        Self {
            color,
            unicode,
            width,
        }
    }
}

impl ColorPreference {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "auto" => Some(ColorPreference::Auto),
            "always" => Some(ColorPreference::Always),
            "never" => Some(ColorPreference::Never),
            _ => None,
        }
    }
}

fn terminal_width() -> Option<usize> {
    std::env::var("COLUMNS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .or_else(|| {
            // Fallback: try to read from stty
            std::process::Command::new("stty")
                .args(["size", "-F", "/dev/stty"])
                .output()
                .ok()
                .and_then(|output| {
                    let s = std::str::from_utf8(&output.stdout).ok()?;
                    let cols = s.split_whitespace().nth(1)?;
                    cols.parse::<usize>().ok()
                })
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_preference_parsing() {
        assert_eq!(ColorPreference::parse("auto"), Some(ColorPreference::Auto));
        assert_eq!(
            ColorPreference::parse("always"),
            Some(ColorPreference::Always)
        );
        assert_eq!(
            ColorPreference::parse("never"),
            Some(ColorPreference::Never)
        );
        assert_eq!(ColorPreference::parse("bad"), None);
    }

    #[test]
    fn detect_respects_no_color() {
        std::env::set_var("NO_COLOR", "1");
        let caps = TerminalCapabilities::detect(ColorPreference::Auto);
        assert!(!caps.color);
        std::env::remove_var("NO_COLOR");
    }
}
