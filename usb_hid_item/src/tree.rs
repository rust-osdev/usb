///! # Report descriptor parser returning data fields
use {
    super::{Item, MainFlags, ParseError},
    core::cell::Cell,
    core::mem,
    core::ops::RangeInclusive,
};

#[derive(Debug)]
pub struct Parser<'a> {
    // Splitting the fields and manually reconstructing the item parser allows
    // avoiding troubles with shared mutable references, lifetimes etc.
    data: &'a [u8],
    index: Cell<usize>,

    // Global state
    usage_page: Cell<u16>,
    logical_min: Cell<i32>,
    logical_max: Cell<i32>,
    physical_min: Cell<i32>,
    physical_max: Cell<i32>,
    report_count: Cell<u32>,
    report_size: Cell<u32>,

    // Local state
    usage_min: Cell<Option<u16>>,
    usage_max: Cell<Option<u16>>,
}

impl<'a> Parser<'a> {
    pub fn iter(&mut self) -> Tree<'a, '_> {
        Tree { inner: self }
    }
}

#[derive(Debug)]
pub struct Tree<'a, 'p> {
    inner: &'p Parser<'a>,
}

impl<'a, 'p> Iterator for Tree<'a, 'p> {
    type Item = Result<Value<'a, 'p>, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        let p = self.inner;
        let mut it = super::Parser {
            data: p.data.get(p.index.get()..)?,
        };
        loop {
            let item = match it.next()? {
                Ok(e) => e,
                Err(e) => return Some(Err(e)),
            };
            p.index.set(p.data.len() - it.data.len());
            match item {
                Item::Collection(ty) => {
                    break Some(Ok(Value::Collection(Collection {
                        ty,
                        inner: Tree { inner: p },
                    })))
                }
                Item::EndCollection => break None,
                Item::UsagePage(pg) => p.usage_page.set(pg),
                Item::Usage16(u) => {
                    let page = p.usage_page.get();
                    break Some(Ok(Value::Usage { page, ids: u..=u }));
                }
                Item::UsageMin(min) => {
                    if let Some(max) = p.usage_max.take() {
                        let page = p.usage_page.get();
                        break Some(Ok(Value::Usage {
                            page,
                            ids: min..=max,
                        }));
                    } else {
                        p.usage_min.set(Some(min));
                    }
                }
                Item::UsageMax(max) => {
                    if let Some(min) = p.usage_min.take() {
                        let page = p.usage_page.get();
                        break Some(Ok(Value::Usage {
                            page,
                            ids: min..=max,
                        }));
                    } else {
                        p.usage_max.set(Some(max));
                    }
                }
                Item::LogicalMin(n) => p.logical_min.set(n),
                Item::LogicalMax(n) => p.logical_max.set(n),
                Item::PhysicalMin(n) => p.physical_min.set(n),
                Item::PhysicalMax(n) => p.physical_max.set(n),
                Item::ReportCount(n) => p.report_count.set(n),
                Item::ReportSize(n) => p.report_size.set(n),
                e @ Item::Input(flags) | e @ Item::Output(flags) => {
                    break Some((|| {
                        let logical_min = p.logical_min.get();
                        let logical_max = p.logical_max.get();
                        let mut physical_min = p.physical_min.get();
                        let mut physical_max = p.physical_max.get();
                        if physical_min == 0 && physical_max == 0 {
                            physical_min = logical_min;
                            physical_max = logical_max;
                        }
                        Ok(Value::Field(Field {
                            is_input: matches!(e, Item::Input(_)),
                            flags,
                            logical_min,
                            logical_max,
                            physical_min,
                            physical_max,
                            report_count: p.report_count.get(),
                            report_size: p.report_size.get(),
                        }))
                    })())
                }
                e => todo!("{:#?}", e),
            };
        }
    }
}

#[derive(Debug)]
pub enum Value<'a, 'p> {
    /// A collection of fields.
    Collection(Collection<'a, 'p>),
    /// A single input/output/feature field.
    Field(Field),
    /// A usage of a field.
    ///
    /// Since a field may have an arbitrary amount of usages, they are returned separately.
    ///
    /// Usages will be returned *before* their corresponding field.
    Usage { page: u16, ids: RangeInclusive<u16> },
}

#[derive(Debug)]
pub struct Collection<'a, 'p> {
    inner: Tree<'a, 'p>,
    pub ty: super::Collection,
}

impl<'a, 'p> Iterator for Collection<'a, 'p> {
    type Item = Result<Value<'a, 'p>, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl Drop for Collection<'_, '_> {
    fn drop(&mut self) {
        while matches!(self.next(), Some(Ok(_))) {}
    }
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
    Parser {
        data,
        index: Default::default(),
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
        I: Iterator<Item = Result<Value<'a, 'a>, ParseError>>,
    {
        match it.next() {
            Some(Ok(Value::Usage { page, ids })) => assert_eq!((page, ids), (p, i)),
            e => panic!("{:#?}", e),
        }
    }

    #[track_caller]
    fn assert_field<'a, I>(it: &mut I, f: Field)
    where
        I: Iterator<Item = Result<Value<'a, 'a>, ParseError>>,
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

        let mut it2 = match it.next() {
            Some(Ok(Value::Collection(c))) => c,
            e => panic!("{:#?}", e),
        };
        assert_eq!(it2.ty, crate::Collection::Application);

        assert_usage(&mut it2, 0x1, 0x1..=0x1);
        let mut it3 = match it2.next() {
            Some(Ok(Value::Collection(c))) => c,
            e => panic!("{:#?}", e),
        };
        assert_eq!(it3.ty, crate::Collection::Physical);

        assert_usage(&mut it3, 0x9, 1..=3);
        assert_field(
            &mut it3,
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
            &mut it3,
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
        assert_usage(&mut it3, 0x1, 0x30..=0x30);
        assert_usage(&mut it3, 0x1, 0x31..=0x31);
        assert_field(
            &mut it3,
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
        assert_usage(&mut it3, 0x1, 0x38..=0x38);
        assert_field(
            &mut it3,
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
        assert!(it3.next().is_none());
        assert!(it2.next().is_none());
        assert!(it.next().is_none());
    }
}
