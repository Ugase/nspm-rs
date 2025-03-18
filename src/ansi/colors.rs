use std::fmt::Display;

pub const ESC: &str = "\x1b[";

pub const RESET: &str = "\x1b[0m";
pub const BLACK: &str = "\x1b[30m";
pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const BLUE: &str = "\x1b[34m";
pub const MAGENTA: &str = "\x1b[35m";
pub const CYAN: &str = "\x1b[36m";
pub const WHITE: &str = "\x1b[37m";
pub const BRIGHT_BLACK: &str = "\x1b[90m";
pub const BRIGHT_RED: &str = "\x1b[91m";
pub const BRIGHT_GREEN: &str = "\x1b[92m";
pub const BRIGHT_YELLOW: &str = "\x1b[93m";
pub const BRIGHT_BLUE: &str = "\x1b[94m";
pub const BRIGHT_MAGENTA: &str = "\x1b[95m";
pub const BRIGHT_CYAN: &str = "\x1b[96m";
pub const BRIGHT_WHITE: &str = "\x1b[97m";

pub const BOLD: &str = "\x1b[1m";

#[derive(Debug, Clone, Copy)]
pub struct AnsiRGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl AnsiRGB {
    pub fn gradient(&self, percent: f64, other: AnsiRGB, thrid: AnsiRGB) -> AnsiRGB {
        let (mut r, mut g, mut b) = (0.0, 0.0, 0.0);
        let (sr, sg, sb, or, og, ob, tr, tg, tb) = (
            self.r as f64,
            self.g as f64,
            self.b as f64,
            other.r as f64,
            other.g as f64,
            other.b as f64,
            thrid.r as f64,
            thrid.g as f64,
            thrid.b as f64,
        );
        if percent < 50.0 {
            r = sr + (or - sr) * (percent / 50.0);
            g = sg + (og - sg) * (percent / 50.0);
            b = sb + (ob - sb) * (percent / 50.0);
        } else if (50..=100).contains(&(percent as u32)) {
            r = or + (tr - or) * ((percent - 50.0) / 50.0);
            g = og + (tg - og) * ((percent - 50.0) / 50.0);
            b = ob + (tb - ob) * ((percent - 50.0) / 50.0);
        }
        AnsiRGB {
            r: r as u8,
            g: g as u8,
            b: b as u8,
        }
    }
}

impl Display for AnsiRGB {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{ESC}38;2;{};{};{}m", self.r, self.g, self.b)
    }
}
