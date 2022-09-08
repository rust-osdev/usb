use core::fmt;

pub struct Hid {
    pub hid_version: u16,
    pub country_code: u8,
    pub num_descriptors: u8,
    pub ty: u8,
    pub len: u16,
}

impl Hid {
    pub(crate) fn from_raw(buf: &[u8]) -> Result<Hid, InvalidHid> {
        if let &[a, b, c, d, e, f, g] = buf {
            Ok(Hid {
                hid_version: u16::from_le_bytes([a, b]),
                country_code: c,
                num_descriptors: d,
                ty: e,
                len: u16::from_le_bytes([f, g]),
            })
        } else {
            Err(InvalidHid::UnexpectedLength)
        }
    }
}

impl fmt::Debug for Hid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [maj, min] = self.hid_version.to_be_bytes();
        f.debug_struct(stringify!(Hid))
            .field("hid_version", &format_args!("{:x}.{:x}", maj, min))
            .field("country_code", &self.country_code)
            .field("num_descriptors", &self.num_descriptors)
            .field("ty", &format_args!("{:#04x}", self.ty))
            .field("len", &self.len)
            .finish()
    }
}

#[derive(Debug)]
pub enum InvalidHid {
    UnexpectedLength,
}
