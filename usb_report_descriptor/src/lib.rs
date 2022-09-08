#![doc = include_str!("../README.md")]
#![cfg_attr(not(test), no_std)]

pub mod usage;

pub use usage::{Usage, UsagePage};

use core::fmt;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum Item<'a> {
    Input(MainFlags),
    Output(MainFlags),
    Collection(Collection),
    Feature(MainFlags),
    EndCollection,

    UsagePage(UsagePage),
    LogicalMin(i32),
    LogicalMax(i32),
    PhysicalMin(i32),
    PhysicalMax(i32),
    UnitExponential(u32),
    Unit(u32),
    ReportSize(u32),
    ReportId(u8),
    ReportCount(u32),
    Push,
    Pop,

    Usage(Usage),
    UsageMin(u32),
    UsageMax(u32),
    DesignatorIndex(u32),
    DesignatorMin(u32),
    DesignatorMax(u32),
    StringIndex(u32),
    StringMin(u32),
    StringMax(u32),
    Delimiter(bool),

    Unknown { tag: u8, data: &'a [u8] },
}

impl<'a> Item<'a> {
    // Main (6.2.2.4)
    const INPUT: u8 = 0x80;
    const OUTPUT: u8 = 0x90;
    const COLLECTION: u8 = 0xa0;
    const FEATURE: u8 = 0xb0;
    const END_COLLECTION: u8 = 0xc0;

    // Global (6.2.2.7)
    const USAGE_PAGE: u8 = 0x04;
    const LOGI_MIN: u8 = 0x14;
    const LOGI_MAX: u8 = 0x24;
    const PHYS_MIN: u8 = 0x34;
    const PHYS_MAX: u8 = 0x44;
    const UNIT_EXP: u8 = 0x54;
    const UNIT: u8 = 0x64;
    const REPORT_SIZE: u8 = 0x74;
    const REPORT_ID: u8 = 0x84;
    const REPORT_COUNT: u8 = 0x94;
    const PUSH: u8 = 0xa4;
    const POP: u8 = 0xb4;

    // Local (6.2.2.8)
    const USAGE: u8 = 0x08;
    const USAGE_MIN: u8 = 0x18;
    const USAGE_MAX: u8 = 0x28;
    const DESIGNATOR_INDEX: u8 = 0x38;
    const DESIGNATOR_MIN: u8 = 0x48;
    const DESIGNATOR_MAX: u8 = 0x58;
    const STRING_INDEX: u8 = 0x78;
    const STRING_MIN: u8 = 0x88;
    const STRING_MAX: u8 = 0x98;
    const DELIMITER: u8 = 0xa8;

    fn parse(data: &'a [u8], usage_page: u16) -> Result<(Self, &'a [u8]), ParseError> {
        use ParseError::*;
        let prefix = *data.get(0).ok_or(Truncated)?;
        let (size, tag);
        if prefix == 0b1111_11_10 {
            // Long item (6.2.2.3)
            size = usize::from(*data.get(1).ok_or(Truncated)?);
            tag = *data.get(2).ok_or(Truncated)?;
        } else {
            // Short item (6.2.2.2)
            size = (1 << (prefix & 0b11)) >> 1;
            tag = prefix & !0b11;
        }
        let d = data.get(1..1 + size).ok_or(Truncated)?;
        let d8 = || {
            d.try_into()
                .map_err(|_| UnexpectedData)
                .map(u8::from_le_bytes)
        };
        let d16u = || {
            Ok(u16::from_le_bytes(match d {
                &[] => [0, 0],
                &[a] => [a, 0],
                &[a, b] => [a, b],
                _ => return Err(UnexpectedData),
            }))
        };
        let d32u = || {
            Ok(u32::from_le_bytes(match d {
                &[] => [0, 0, 0, 0],
                &[a] => [a, 0, 0, 0],
                &[a, b] => [a, b, 0, 0],
                &[a, b, c] => [a, b, c, 0],
                &[a, b, c, d] => [a, b, c, d],
                _ => return Err(UnexpectedData),
            }))
        };
        let d32i = || {
            Ok(match d {
                &[] => 0,
                &[a] => i8::from_le_bytes([a]) as _,
                &[a, b] => i16::from_le_bytes([a, b]) as _,
                &[a, b, c] => i32::from_le_bytes([a, b, c, (c as i8 >> 7) as _]),
                &[a, b, c, d] => i32::from_le_bytes([a, b, c, d]),
                _ => return Err(UnexpectedData),
            })
        };
        let d_empty = |e| d.is_empty().then(|| e).ok_or(UnexpectedData);

        let item = match tag {
            Self::INPUT => Self::Input(MainFlags(d32u()?)),
            Self::OUTPUT => Self::Output(MainFlags(d32u()?)),
            Self::COLLECTION => Self::Collection(Collection::from_raw(d8()?)),
            Self::FEATURE => Self::Feature(MainFlags(d32u()?)),
            Self::END_COLLECTION => d_empty(Self::EndCollection)?,

            Self::USAGE_PAGE => Self::UsagePage(UsagePage::from_raw(d16u()?)),
            Self::LOGI_MIN => Self::LogicalMin(d32i()?),
            Self::LOGI_MAX => Self::LogicalMax(d32i()?),
            Self::PHYS_MIN => Self::PhysicalMin(d32i()?),
            Self::PHYS_MAX => Self::PhysicalMax(d32i()?),
            Self::UNIT_EXP => Self::UnitExponential(d32u()?),
            Self::UNIT => Self::Unit(d32u()?),
            Self::REPORT_SIZE => Self::ReportSize(d32u()?),
            Self::REPORT_ID => Self::ReportId(d8()?),
            Self::REPORT_COUNT => Self::ReportCount(d32u()?),
            Self::PUSH => Self::Push,
            Self::POP => Self::Pop,

            Self::USAGE => Self::Usage(match d {
                &[] => Usage::from_raw(usage_page, 0),
                &[a] => Usage::from_raw(usage_page, u16::from_le_bytes([a, 0])),
                &[a, b] => Usage::from_raw(usage_page, u16::from_le_bytes([a, b])),
                &[a, b, c] => {
                    Usage::from_raw(u16::from_le_bytes([c, 0]), u16::from_le_bytes([a, b]))
                }
                &[a, b, c, d] => {
                    Usage::from_raw(u16::from_le_bytes([c, d]), u16::from_le_bytes([a, b]))
                }
                _ => return Err(UnexpectedData),
            }),
            Self::USAGE_MIN => Self::UsageMin(d32u()?),
            Self::USAGE_MAX => Self::UsageMax(d32u()?),
            Self::DESIGNATOR_INDEX => Self::DesignatorIndex(d32u()?),
            Self::DESIGNATOR_MIN => Self::DesignatorMin(d32u()?),
            Self::DESIGNATOR_MAX => Self::DesignatorMax(d32u()?),
            Self::STRING_INDEX => Self::StringIndex(d32u()?),
            Self::STRING_MIN => Self::StringMin(d32u()?),
            Self::STRING_MAX => Self::StringMax(d32u()?),
            Self::DELIMITER => Self::Delimiter(match d {
                &[0] => true,
                &[1] => false,
                _ => return Err(UnexpectedData),
            }),

            _ => Self::Unknown { tag, data: d },
        };

        Ok((item, &data[1 + size..]))
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MainFlags(pub u32);

macro_rules! flags {
    { $($flag:ident $bit:literal)* } => {
        impl MainFlags {
            $(
                pub fn $flag(&self) -> bool {
                    self.0 & 1 << $bit != 0
                }
            )*
        }

        impl fmt::Debug for MainFlags {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct(stringify!(MainFlags))
                    $(.field(stringify!($flag), &self.$flag()))*
                    .finish_non_exhaustive()
            }
        }
    };
}

flags! {
    constant 0
    variable 1
    relative 2
    wrap 3
    non_linear 4
    no_preferred 5
    null_state 6
    volatile 7
    buffered_bytes 8
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum Collection {
    Physical,
    Application,
    Logical,
    Report,
    NamedArray,
    UsageSwitch,
    UsageModifier,
    Unknown(u8),
}

impl Collection {
    fn from_raw(raw: u8) -> Self {
        match raw {
            0x00 => Self::Physical,
            0x01 => Self::Application,
            0x02 => Self::Logical,
            0x03 => Self::Report,
            0x04 => Self::NamedArray,
            0x05 => Self::UsageSwitch,
            0x06 => Self::UsageModifier,
            r => Self::Unknown(r),
        }
    }
}

#[derive(Debug)]
pub enum ParseError {
    Truncated,
    UnexpectedData,
}

pub struct Parser<'a> {
    data: &'a [u8],
    usage_page: u16,
}

impl<'a> Iterator for Parser<'a> {
    type Item = Result<Item<'a>, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        (!self.data.is_empty()).then(|| {
            let e;
            (e, self.data) = Item::parse(self.data, self.usage_page)?;
            match &e {
                Item::UsagePage(p) => self.usage_page = p.as_raw(),
                _ => {}
            }
            Ok(e)
        })
    }
}

pub fn parse(data: &[u8]) -> Parser<'_> {
    Parser {
        data,
        usage_page: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // usb/dev-hid.c
    const QEMU_USB_TABLET: &[u8] = &[
        0x05, 0x01, 0x09, 0x02, 0xa1, 0x01, 0x09, 0x01, 0xa1, 0x00, 0x05, 0x09, 0x19, 0x01, 0x29,
        0x03, 0x15, 0x00, 0x25, 0x01, 0x95, 0x03, 0x75, 0x01, 0x81, 0x02, 0x95, 0x01, 0x75, 0x05,
        0x81, 0x01, 0x05, 0x01, 0x09, 0x30, 0x09, 0x31, 0x15, 0x00, 0x26, 0xff, 0x7f, 0x35, 0x00,
        0x46, 0xff, 0x7f, 0x75, 0x10, 0x95, 0x02, 0x81, 0x02, 0x05, 0x01, 0x09, 0x38, 0x15, 0x81,
        0x25, 0x7f, 0x35, 0x00, 0x45, 0x00, 0x75, 0x08, 0x95, 0x01, 0x81, 0x06, 0xc0, 0xc0,
    ];

    #[track_caller]
    fn tk(it: &mut Parser<'_>, item: Item<'_>) {
        assert_eq!(it.next().map(Result::unwrap), Some(item));
    }

    #[test]
    fn qemu_usb_tablet() {
        let mut it = parse(QEMU_USB_TABLET);
        let it = &mut it;
        tk(it, Item::UsagePage(UsagePage::GenericDesktop));
        tk(it, Item::Usage(Usage::GenericDesktop(usage::generic_desktop::Usage::Mouse)));
        tk(it, Item::Collection(Collection::Application));
        tk(it, Item::Usage(Usage::GenericDesktop(usage::generic_desktop::Usage::Pointer)));
        tk(it, Item::Collection(Collection::Physical));
        tk(it, Item::UsagePage(UsagePage::Button));
        tk(it, Item::UsageMin(1));
        tk(it, Item::UsageMax(3));
        tk(it, Item::LogicalMin(0));
        tk(it, Item::LogicalMax(1));
        tk(it, Item::ReportCount(3));
        tk(it, Item::ReportSize(1));
        tk(it, Item::Input(MainFlags(0b010))); // absolute, variable, data
        tk(it, Item::ReportCount(1));
        tk(it, Item::ReportSize(5));
        tk(it, Item::Input(MainFlags(0b1))); // constant
        tk(it, Item::UsagePage(UsagePage::GenericDesktop));
        tk(it, Item::Usage(Usage::GenericDesktop(usage::generic_desktop::Usage::X)));
        tk(it, Item::Usage(Usage::GenericDesktop(usage::generic_desktop::Usage::Y)));
        tk(it, Item::LogicalMin(0));
        tk(it, Item::LogicalMax(0x7fff));
        tk(it, Item::PhysicalMin(0));
        tk(it, Item::PhysicalMax(0x7fff));
        tk(it, Item::ReportSize(16));
        tk(it, Item::ReportCount(2));
        tk(it, Item::Input(MainFlags(0b010))); // absolute, variable, data
        tk(it, Item::UsagePage(UsagePage::GenericDesktop));
        tk(it, Item::Usage(Usage::GenericDesktop(usage::generic_desktop::Usage::Wheel)));
        tk(it, Item::LogicalMin(-0x7f));
        tk(it, Item::LogicalMax(0x7f));
        tk(it, Item::PhysicalMin(0));
        tk(it, Item::PhysicalMax(0));
        tk(it, Item::ReportSize(8));
        tk(it, Item::ReportCount(1));
        tk(it, Item::Input(MainFlags(0b110))); // relative, variable, data
        tk(it, Item::EndCollection);
        tk(it, Item::EndCollection);
        assert!(it.next().is_none());
    }
}
