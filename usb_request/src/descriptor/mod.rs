mod configuration;
mod device;
mod endpoint;
mod hid;
mod interface;
mod report;
mod string;

pub use configuration::*;
pub use device::*;
pub use endpoint::*;
pub use hid::*;
pub use interface::*;
pub use report::*;
pub use string::*;

use core::mem;

#[derive(Debug)]
pub enum GetDescriptor {
    Device,
    Configuration { index: u8 },
    String { index: u8 },
    Report,
}

pub(crate) const DEVICE: u8 = 0x1;
pub(crate) const CONFIGURATION: u8 = 0x2;
pub(crate) const STRING: u8 = 0x3;
pub(crate) const INTERFACE: u8 = 0x4;
pub(crate) const ENDPOINT: u8 = 0x5;
#[allow(dead_code)]
pub(crate) const DEVICE_QUALIFIER: u8 = 0x6;
#[allow(dead_code)]
pub(crate) const OTHER_SPEED_CONFIGURATION: u8 = 0x7;
#[allow(dead_code)]
pub(crate) const INTERFACE_POWER: u8 = 0x8;

pub(crate) const HID: u8 = 0x21;
pub(crate) const REPORT: u8 = 0x22;
#[allow(dead_code)]
pub(crate) const PHYSICAL: u8 = 0x23;

#[derive(Debug)]
pub enum Descriptor<'a> {
    Device(Device),
    Configuration(Configuration),
    String(StringIter<'a>),
    Interface(Interface),
    Endpoint(Endpoint),
    Hid(Hid),
    Report(Report<'a>),
    Unknown { ty: u8, data: &'a [u8] },
}

macro_rules! into {
    ($v:ident $f:ident $t:ty) => {
        pub fn $f(self) -> Option<$t> {
            match self {
                Self::$v(v) => Some(v),
                _ => None,
            }
        }
    };
}

impl<'a> Descriptor<'a> {
    into!(Device into_device Device);
    into!(String into_string StringIter<'a>);
    into!(Configuration into_configuration Configuration);
}

#[derive(Debug)]
pub struct StringIter<'a>(&'a [[u8; 2]]);

impl<'a> StringIter<'a> {
    pub(crate) fn from_raw(data: &'a [u8]) -> Result<StringIter, InvalidString> {
        let (s, rem) = data.as_chunks();
        rem.is_empty()
            .then(|| Self(s))
            .ok_or(InvalidString::UnexpectedLength)
    }
}

impl Iterator for StringIter<'_> {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.split_first().map(|(c, s)| {
            self.0 = s;
            u16::from_le_bytes(*c)
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl ExactSizeIterator for StringIter<'_> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

#[derive(Debug)]
pub enum InvalidString {
    UnexpectedLength,
}

pub fn decode(buf: &[u8]) -> Iter<'_> {
    Iter { buf }
}

pub struct Iter<'a> {
    buf: &'a [u8],
}

impl<'a> Iterator for Iter<'a> {
    type Item = Result<Descriptor<'a>, InvalidDescriptor>;

    fn next(&mut self) -> Option<Self::Item> {
        (!self.buf.is_empty()).then(|| {
            let buf = mem::take(&mut self.buf);
            let l = buf[0];
            if l < 2 || usize::from(l) > buf.len() {
                return Err(InvalidDescriptor::Truncated { length: l.max(2) });
            }
            let b = &buf[2..usize::from(l)];
            let r = match buf[1] {
                DEVICE => {
                    Descriptor::Device(Device::from_raw(b).map_err(InvalidDescriptor::Device)?)
                }
                CONFIGURATION => Descriptor::Configuration(
                    Configuration::from_raw(b).map_err(InvalidDescriptor::Configuration)?,
                ),
                STRING => {
                    Descriptor::String(StringIter::from_raw(b).map_err(InvalidDescriptor::String)?)
                }
                INTERFACE => Descriptor::Interface(
                    Interface::from_raw(b).map_err(InvalidDescriptor::Interface)?,
                ),
                ENDPOINT => Descriptor::Endpoint(
                    Endpoint::from_raw(b).map_err(InvalidDescriptor::Endpoint)?,
                ),
                HID => Descriptor::Hid(Hid::from_raw(b).map_err(InvalidDescriptor::Hid)?),
                REPORT => {
                    Descriptor::Report(Report::from_raw(b).map_err(InvalidDescriptor::Report)?)
                }
                ty => Descriptor::Unknown { ty, data: b },
            };
            self.buf = &buf[usize::from(l)..];
            Ok(r)
        })
    }
}

#[derive(Debug)]
pub enum InvalidDescriptor {
    Truncated { length: u8 },
    Device(InvalidDevice),
    Configuration(InvalidConfiguration),
    String(InvalidString),
    Interface(InvalidInterface),
    Endpoint(InvalidEndpoint),
    Hid(InvalidHid),
    Report(InvalidReport),
}
