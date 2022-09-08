#[derive(Debug)]
pub struct Interface {
    pub number: u8,
    pub alternate_setting: u8,
    pub num_endpoints: u8,
    pub class: u8,
    pub subclass: u8,
    pub protocol: u8,
    pub index: u8,
}

impl Interface {
    pub(crate) fn from_raw(buf: &[u8]) -> Result<Self, InvalidInterface> {
        if let &[a, b, c, d, e, f, g] = buf {
            Ok(Interface {
                number: a,
                alternate_setting: b,
                num_endpoints: c,
                class: d,
                subclass: e,
                protocol: f,
                index: g,
            })
        } else {
            Err(InvalidInterface::UnexpectedLength)
        }
    }
}

#[derive(Debug)]
pub enum InvalidInterface {
    UnexpectedLength,
}
