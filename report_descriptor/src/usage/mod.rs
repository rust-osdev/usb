//! # References
//!
//! * https://www.usb.org/sites/default/files/hut1_3_0.pdf

macro_rules! usage {
    { [$page:literal] $($i:literal $v:ident)* } => {
        pub const PAGE: u16 = $page;

        #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[non_exhaustive]
        pub enum Usage {
            $($v,)*
            Unknown(u16),
        }

        pub(crate) fn from_raw(raw: u16) -> Usage {
            use Usage::*;
            match raw {
                $($i => $v,)*
                r => Unknown(r),
            }
        }
    };
}

pub mod generic_desktop;
pub mod button;

macro_rules! page {
    { $($v:ident $m:ident)* } => {
        #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[non_exhaustive]
        pub enum UsagePage {
            $($v,)*
            Unknown(u16),
        }

        impl UsagePage {
            pub(crate) fn from_raw(raw: u16) -> Self {
                match raw {
                    $($m::PAGE => Self::$v,)*
                    r => Self::Unknown(r),
                }
            }

            pub(crate) fn as_raw(&self) -> u16 {
                match self {
                    $(Self::$v => $m::PAGE,)*
                    Self::Unknown(n) => *n,
                }
            }
        }

        #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[non_exhaustive]
        pub enum Usage {
            $($v($m::Usage),)*
            Unknown(u16, u16),
        }

        impl Usage {
            pub(crate) fn from_raw(page: u16, id: u16) -> Self {
                match page {
                    $($m::PAGE => Self::$v($m::from_raw(id)),)*
                    _ => Self::Unknown(page, id),
                }
            }
        }
    };
}

page! {
    GenericDesktop generic_desktop
    Button button
}
