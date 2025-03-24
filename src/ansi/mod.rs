use std::fmt::Display;
use std::io::{Write, stdout};
pub mod colors;
pub mod constants;

pub const ESC: &str = "\x1b[";
pub const CLEAR: &str = "\x1b[H\x1b[2J\x1b[3J";
#[inline(always)]
fn ansi(seq: &str) -> String {
    ESC.to_owned() + seq
}

pub fn clear_line() {
    let mut buf = stdout();
    print!("{}{}{}", Csi::CPL, Csi::CNL, Csi::El(EL::EL2));
    let _ = buf.flush();
}

pub enum EL {
    //EL0,
    //EL1,
    EL2,
}

pub enum Csi {
    CNL,
    CPL,
    El(EL),
    Hide,
    Show,
}

impl Display for Csi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Csi::CPL => write!(f, "{}", ansi("F")),
            Csi::CNL => write!(f, "{}", ansi("E")),
            Csi::El(e) => match e {
                //EL::EL0 => return ansi("K"),
                //EL::EL1 => return ansi("1K"),
                EL::EL2 => write!(f, "{}", ansi("2K")),
            },
            Csi::Hide => write!(f, "{}", ansi("?25l")),
            Csi::Show => write!(f, "{}", ansi("?25h")),
        }
    }
}
