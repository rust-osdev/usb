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
    };
}

page! {
    GenericDesktop generic_desktop
    Button button
}

#[derive(Debug)]
pub struct UnknownUsage;

#[derive(Debug)]
pub struct UnknownPage;
