#![doc = include_str!("../README.md")]
#![cfg_attr(not(test), no_std)]

pub mod item;

pub use item::{Collection, MainFlags};

use {
    item::Item,
    core::cell::Cell,
    core::ops::RangeInclusive,
};

#[derive(Debug)]
pub struct Parser<'a> {
    // Manually reconstructing the item parser allows avoiding troubles with shared mutable
    // references, lifetimes etc.
    data: Cell<&'a [u8]>,
}

impl<'a> Parser<'a> {
    pub fn iter(&mut self) -> StackFrame<'a, '_> {
        StackFrame::new(self)
    }
}

#[derive(Debug)]
pub struct StackFrame<'a, 'p> {
    inner: &'p Parser<'a>,

    // Global state
    usage_page: u16,
    logical_min: i32,
    logical_max: i32,
    physical_min: i32,
    physical_max: i32,
    report_count: u32,
    report_size: u32,

    // Local state
    usage_min: Option<u16>,
    usage_max: Option<u16>,
}

impl<'a, 'p> StackFrame<'a, 'p> {
    fn new(inner: &'p Parser<'a>) -> Self {
        Self {
            inner,
            usage_page: Default::default(),
            usage_min: Default::default(),
            usage_max: Default::default(),
            logical_min: Default::default(),
            logical_max: Default::default(),
            physical_min: Default::default(),
            physical_max: Default::default(),
            report_count: Default::default(),
            report_size: Default::default(),
        }
    }

    fn duplicate(&self) -> Self {
        Self {
            inner: self.inner,
            usage_page: self.usage_page,
            usage_min: self.usage_min,
            usage_max: self.usage_max,
            logical_min: self.logical_min,
            logical_max: self.logical_max,
            physical_min: self.physical_min,
            physical_max: self.physical_max,
            report_count: self.report_count,
            report_size: self.report_size,
        }
    }
}

impl<'a, 'p> Iterator for StackFrame<'a, 'p> {
    type Item = Result<Value<'a, 'p>, ParseError<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut it = item::Parser {
            data: self.inner.data.get(),
        };
        loop {
            let item = match it.next()? {
                Ok(e) => e,
                Err(e) => return Some(Err(ParseError::from_item(e))),
            };
            self.inner.data.set(it.data);
            match item {
                Item::Collection(ty) => break Some(Ok(Value::Collection(ty))),
                Item::EndCollection => break Some(Ok(Value::EndCollection)),
                Item::UsagePage(p) => self.usage_page = p,
                Item::Usage16(u) => {
                    break Some(Ok(Value::Usage {
                        page: self.usage_page,
                        ids: u..=u,
                    }))
                }
                Item::UsageMin(min) => {
                    if let Some(max) = self.usage_max.take() {
                        break Some(Ok(Value::Usage {
                            page: self.usage_page,
                            ids: min..=max,
                        }));
                    } else {
                        self.usage_min = Some(min);
                    }
                }
                Item::UsageMax(max) => {
                    if let Some(min) = self.usage_min.take() {
                        break Some(Ok(Value::Usage {
                            page: self.usage_page,
                            ids: min..=max,
                        }));
                    } else {
                        self.usage_max = Some(max);
                    }
                }
                Item::LogicalMin(n) => self.logical_min = n,
                Item::LogicalMax(n) => self.logical_max = n,
                Item::PhysicalMin(n) => self.physical_min = n,
                Item::PhysicalMax(n) => self.physical_max = n,
                Item::ReportCount(n) => self.report_count = n,
                Item::ReportSize(n) => self.report_size = n,
                e @ Item::Input(flags) | e @ Item::Output(flags) => {
                    let mut physical_min = self.physical_min;
                    let mut physical_max = self.physical_max;
                    if physical_min == 0 && physical_max == 0 {
                        physical_min = self.logical_min;
                        physical_max = self.logical_max;
                    }
                    break Some(Ok(Value::Field(Field {
                        is_input: matches!(e, Item::Input(_)),
                        flags,
                        logical_min: self.logical_min,
                        logical_max: self.logical_max,
                        physical_min,
                        physical_max,
                        report_count: self.report_count,
                        report_size: self.report_size,
                    })));
                }
                Item::ReportId(_) => {} // TODO
                Item::Push => break Some(Ok(Value::StackFrame(self.duplicate()))),
                Item::Pop => break None,
                Item::Unit(_) => {}         // TODO
                Item::UnitExponent(_) => {} // TODO
                e => break Some(Err(ParseError::UnexpectedItem(e))),
            };
        }
    }
}

impl Drop for StackFrame<'_, '_> {
    fn drop(&mut self) {
        while matches!(self.next(), Some(Ok(_))) {}
    }
}

#[derive(Debug)]
pub enum ParseError<'a> {
    /// An item is longer than the amount of bytes remaining in the buffer.
    Truncated,
    /// An item has an unexpected data value.
    UnexpectedData,
    UnexpectedItem(Item<'a>),
}

impl ParseError<'_> {
    fn from_item(e: item::ParseError) -> Self {
        match e {
            item::ParseError::Truncated => Self::Truncated,
            item::ParseError::UnexpectedData => Self::UnexpectedData,
        }
    }
}

#[derive(Debug)]
pub enum Value<'a, 'p> {
    /// The start of a collection of fields.
    Collection(Collection),
    /// The end of a collection of fields.
    EndCollection,
    /// A single input/output/feature field.
    Field(Field),
    /// A usage of a field.
    ///
    /// Since a field may have an arbitrary amount of usages, they are returned separately.
    ///
    /// Usages will be returned *before* their corresponding field.
    Usage { page: u16, ids: RangeInclusive<u16> },
    /// A stack frame with global state.
    ///
    /// This is returned after a Push item.
    /// When `next` returns `None` either a Pop item was encountered or the end of the descriptor
    /// was reached.
    StackFrame(StackFrame<'a, 'p>),
}

#[derive(Debug)]
pub struct Field {
    /// Whether this is an input or output field.
    pub is_input: bool,
    /// Flags belonging to this field.
    pub flags: MainFlags,
    /// The minimum value this field can contain.
    pub logical_min: i32,
    /// The maximum value this field can contain.
    pub logical_max: i32,
    /// The maximum physical value this field can represent.
    pub physical_min: i32,
    /// The minimum physical value this field can represent.
    pub physical_max: i32,
    /// How many times this field repeats.
    pub report_count: u32,
    /// The size of this field in bits.
    pub report_size: u32,
}

impl Field {
    /// Try to extract a field's value from a report.
    ///
    /// This only extracts a single field, i.e. it ignores `report_count`.
    pub fn extract_u32(&self, report: &[u8], offset: u32) -> Option<u32> {
        if self.report_size > 32 {
            return None;
        }
        let (start, end) = (offset, offset + self.report_size);
        let (start_i, end_i) = (start / 8, (end + 7) / 8);
        let mut v = 0;
        for (i, &b) in report.get(start_i as _..end_i as _)?.iter().enumerate() {
            v |= u32::from(b) << i * 8 >> start % 8;
        }
        v %= 1 << self.report_size;
        Some(v)
    }

    /// Try to extract a field's value from a report.
    ///
    /// This only extracts a single field, i.e. it ignores `report_count`.
    pub fn extract_i32(&self, report: &[u8], offset: u32) -> Option<i32> {
        self.extract_u32(report, offset).map(|n| {
            // sign-extend
            (n as i32) << 32 - self.report_size >> 32 - self.report_size
        })
    }
}

pub fn parse(data: &[u8]) -> Parser<'_> {
    Parser { data: data.into() }
}

#[cfg(test)]
mod test {
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
    fn assert_usage<'a, I>(it: &mut I, p: u16, i: RangeInclusive<u16>)
    where
        I: Iterator<Item = Result<Value<'a, 'a>, ParseError<'a>>>,
    {
        match it.next() {
            Some(Ok(Value::Usage { page, ids })) => assert_eq!((page, ids), (p, i)),
            e => panic!("{:#?}", e),
        }
    }

    #[track_caller]
    fn assert_field<'a, I>(it: &mut I, f: Field)
    where
        I: Iterator<Item = Result<Value<'a, 'a>, ParseError<'a>>>,
    {
        match it.next() {
            Some(Ok(Value::Field(v))) => {
                assert_eq!(v.flags, f.flags);
                assert_eq!(v.logical_min, f.logical_min);
                assert_eq!(v.logical_max, f.logical_max);
                assert_eq!(v.physical_min, f.physical_min);
                assert_eq!(v.physical_max, f.physical_max);
                assert_eq!(v.report_count, f.report_count);
                assert_eq!(v.report_size, f.report_size);
            }
            e => panic!("{:#?}", e),
        }
    }

    #[test]
    fn qemu_usb_tablet() {
        let mut it = parse(QEMU_USB_TABLET);
        let mut it = it.iter();
        assert_usage(&mut it, 0x1, 0x2..=0x2);

        assert!(matches!(
            it.next(),
            Some(Ok(Value::Collection(Collection::Application)))
        ));

        assert_usage(&mut it, 0x1, 0x1..=0x1);
        assert!(matches!(
            it.next(),
            Some(Ok(Value::Collection(Collection::Physical)))
        ));

        assert_usage(&mut it, 0x9, 1..=3);
        assert_field(
            &mut it,
            Field {
                is_input: true,
                flags: MainFlags(0b010), // absolute, variable, data
                logical_min: 0,
                logical_max: 1,
                physical_min: 0,
                physical_max: 1,
                report_count: 3,
                report_size: 1,
            },
        );
        assert_field(
            &mut it,
            Field {
                is_input: true,
                flags: MainFlags(0b1), // constant
                logical_min: 0,
                logical_max: 1,
                physical_min: 0,
                physical_max: 1,
                report_count: 1,
                report_size: 5,
            },
        );
        assert_usage(&mut it, 0x1, 0x30..=0x30);
        assert_usage(&mut it, 0x1, 0x31..=0x31);
        assert_field(
            &mut it,
            Field {
                is_input: true,
                flags: MainFlags(0b010), // absolute, variable, data
                logical_min: 0,
                logical_max: 0x7fff,
                physical_min: 0,
                physical_max: 0x7fff,
                report_count: 2,
                report_size: 16,
            },
        );
        assert_usage(&mut it, 0x1, 0x38..=0x38);
        assert_field(
            &mut it,
            Field {
                is_input: true,
                flags: MainFlags(0b110), // relative, variable, data
                logical_min: -0x7f,
                logical_max: 0x7f,
                physical_min: -0x7f,
                physical_max: 0x7f,
                report_count: 1,
                report_size: 8,
            },
        );
        assert!(matches!(it.next(), Some(Ok(Value::EndCollection))));
        assert!(matches!(it.next(), Some(Ok(Value::EndCollection))));
        assert!(it.next().is_none());
    }

    #[test]
    fn push() {
        // Not a real descriptor, but the only one I could find with a Push item is excessively
        // long.
        const PUSH: &[u8] = &[
            0x05, 0x01, // UsagePage(1)
            0x15, 0x13, // LogicalMin(0x13)
            0x25, 0x37, // LogicalMax(0x37)
            0x95, 0x07, // ReportCount(7)
            0x75, 0x05, // ReportSize(5)
            0x09, 0x04, // Usage(4)
            0x80, // Input
            0xa4, // Push
            0x05, 0x03, // UsagePage(3)
            0x09, 0x02, // Usage(2)
            0x16, 0xde, 0x00, // LogicalMin(0xde)
            0x26, 0xad, 0x00, // LogicalMax(0xad)
            0x95, 0x09, // ReportCount(9)
            0x75, 0x02, // ReportSize(2)
            0x80, // Input
            0x09, 0x02, // Usage(2)
            0xb4, // Pop
            0x09, 0x02, // Usage(2)
            0x80, // Input
        ];
        let mut it = parse(PUSH);
        let mut it = it.iter();
        assert_usage(&mut it, 1, 4..=4);
        assert_field(
            &mut it,
            Field {
                is_input: true,
                flags: MainFlags(0),
                logical_min: 0x13,
                logical_max: 0x37,
                physical_min: 0x13,
                physical_max: 0x37,
                report_count: 7,
                report_size: 5,
            },
        );
        let mut it2 = match it.next() {
            Some(Ok(Value::StackFrame(f))) => f,
            e => panic!("{:#?}", e),
        };
        assert_usage(&mut it2, 3, 2..=2);
        assert_field(
            &mut it2,
            Field {
                is_input: true,
                flags: MainFlags(0),
                logical_min: 0xde,
                logical_max: 0xad,
                physical_min: 0xde,
                physical_max: 0xad,
                report_count: 9,
                report_size: 2,
            },
        );
        assert_usage(&mut it2, 3, 2..=2);
        assert!(it2.next().is_none());
        assert_usage(&mut it, 1, 2..=2);
        assert_field(
            &mut it,
            Field {
                is_input: true,
                flags: MainFlags(0),
                logical_min: 0x13,
                logical_max: 0x37,
                physical_min: 0x13,
                physical_max: 0x37,
                report_count: 7,
                report_size: 5,
            },
        );
    }
}
