//!
//! ## References
//!
//! * <https://www.usb.org/sites/default/files/hut1_3_0.pdf>

#![no_std]

macro_rules! usage {
    { [$page:literal] $($i:literal $v:ident)* } => {
        pub const PAGE: u16 = $page;

        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[non_exhaustive]
        pub enum Usage {
            $($v,)*
        }

        impl TryFrom<u16> for Usage {
            type Error = super::UnknownUsage;

            fn try_from(n: u16) -> Result<Self, Self::Error> {
                Ok(match n {
                    $($i => Self::$v,)*
                    _ => return Err(super::UnknownUsage),
                })
            }
        }
    };
}

macro_rules! page {
    { $($v:ident $m:ident)* } => {
        $(pub mod $m;)*

        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[non_exhaustive]
        pub enum UsagePage {
            $($v,)*
        }

        impl TryFrom<u16> for UsagePage {
            type Error = UnknownPage;

            fn try_from(n: u16) -> Result<Self, Self::Error> {
                Ok(match n {
                    $($m::PAGE => Self::$v,)*
                    _ => return Err(UnknownPage),
                })
            }
        }

        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[non_exhaustive]
        pub enum Usage {
            $($v($m::Usage),)*
        }

        impl TryFrom<(u16, u16)> for Usage {
            type Error = UnknownUsage;

            fn try_from((page, usage): (u16, u16)) -> Result<Self, Self::Error> {
                Ok(match page {
                    $($m::PAGE => Self::$v($m::Usage::try_from(usage).map_err(|_| UnknownUsage)?),)*
                    _ => return Err(UnknownUsage),
                })
            }
        }
    };
}

page! {
    GenericDesktop generic_desktop
    Keyboard keyboard
    Button button
}

#[derive(Debug)]
pub struct UnknownUsage;

#[derive(Debug)]
pub struct UnknownPage;
