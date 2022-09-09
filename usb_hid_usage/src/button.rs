use core::num::NonZeroU16;

pub const PAGE: u16 = 0x09;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Usage {
    NoButton,
    Button(NonZeroU16),
}

impl TryFrom<u16> for Usage {
    type Error = super::UnknownUsage;

    fn try_from(n: u16) -> Result<Self, Self::Error> {
        Ok(NonZeroU16::new(n).map_or(Self::NoButton, Self::Button))
    }
}
