pub mod colors;

pub const ESC: &str = "\x1b[";
pub const CLEAR: &str = "\x1b[H\x1b[2J\x1b[3J";
#[inline(always)]
fn ansi(seq: &str) -> String {
    ESC.to_owned() + seq
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
}

impl Csi {
    pub fn ansi(self) -> String {
        match self {
            Csi::CPL => ansi("F"),
            Csi::CNL => ansi("E"),
            Csi::El(e) => match e {
                //EL::EL0 => return ansi("K"),
                //EL::EL1 => return ansi("1K"),
                EL::EL2 => return ansi("2K"),
            },
        }
    }
}
