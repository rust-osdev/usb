use core::fmt;

pub struct Report<'a> {
    pub data: &'a [u8],
}

impl<'a> Report<'a> {
    pub(crate) fn from_raw(data: &'a [u8]) -> Result<Self, InvalidReport> {
        Ok(Self { data })
    }
}

impl fmt::Debug for Report<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(stringify!(Report))
            .field("data", &format_args!("{:02x?}", self.data))
            .finish()
    }
}

#[derive(Debug)]
pub enum InvalidReport {}
