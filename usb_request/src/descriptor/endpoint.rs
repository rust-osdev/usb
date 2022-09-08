use core::fmt;

#[derive(Debug)]
pub struct Endpoint {
    /// The address of the endpoint on the USB device described by this descriptor.
    pub address: EndpointAddress,
    pub attributes: EndpointAttributes,
    pub max_packet_size: u16,
    pub interval: u8,
}

impl Endpoint {
    pub(crate) fn from_raw(buf: &[u8]) -> Result<Endpoint, InvalidEndpoint> {
        if let &[a, b, c, d, e] = buf {
            Ok(Endpoint {
                address: EndpointAddress::from_raw(a).ok_or(InvalidEndpoint::InvalidAddress)?,
                attributes: EndpointAttributes::from_raw(b)
                    .ok_or(InvalidEndpoint::InvalidAttributes)?,
                max_packet_size: u16::from_le_bytes([c, d]),
                interval: e,
            })
        } else {
            Err(InvalidEndpoint::UnexpectedLength)
        }
    }
}

pub struct EndpointAddress(u8);

impl EndpointAddress {
    pub fn direction(&self) -> Direction {
        if self.0 & 1 << 7 == 0 {
            Direction::Out
        } else {
            Direction::In
        }
    }

    pub fn number(&self) -> EndpointNumber {
        use EndpointNumber::*;
        match self.0 & 0xf {
            1 => N1,
            2 => N2,
            3 => N3,
            4 => N4,
            5 => N5,
            6 => N6,
            7 => N7,
            8 => N8,
            9 => N9,
            10 => N10,
            11 => N11,
            12 => N12,
            13 => N13,
            14 => N14,
            15 => N15,
            _ => unreachable!(),
        }
    }

    fn from_raw(n: u8) -> Option<Self> {
        (1..=15).contains(&(n & 0xf)).then(|| Self(n))
    }
}

impl fmt::Debug for EndpointAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(stringify!(EndpointAddress))
            .field("direction", &self.direction())
            .field("number", &self.number())
            .finish()
    }
}

#[derive(Debug)]
pub enum EndpointNumber {
    N1,
    N2,
    N3,
    N4,
    N5,
    N6,
    N7,
    N8,
    N9,
    N10,
    N11,
    N12,
    N13,
    N14,
    N15,
}

impl From<EndpointNumber> for usize {
    fn from(n: EndpointNumber) -> usize {
        use EndpointNumber::*;
        match n {
            N1 => 1,
            N2 => 2,
            N3 => 3,
            N4 => 4,
            N5 => 5,
            N6 => 6,
            N7 => 7,
            N8 => 8,
            N9 => 9,
            N10 => 10,
            N11 => 11,
            N12 => 12,
            N13 => 13,
            N14 => 14,
            N15 => 15,
        }
    }
}

pub struct EndpointAttributes(u8);

impl EndpointAttributes {
    pub fn usage(&self) -> EndpointUsage {
        match self.0 >> 4 & 0x3 {
            0 => EndpointUsage::Data,
            1 => EndpointUsage::Feedback,
            2 => EndpointUsage::Implicit,
            _ => unreachable!(),
        }
    }

    pub fn sync(&self) -> EndpointSync {
        match self.0 >> 2 & 0x3 {
            0 => EndpointSync::None,
            1 => EndpointSync::Async,
            2 => EndpointSync::Adapt,
            3 => EndpointSync::Sync,
            _ => unreachable!(),
        }
    }

    pub fn transfer(&self) -> EndpointTransfer {
        match self.0 & 0x3 {
            0 => EndpointTransfer::Control,
            1 => EndpointTransfer::Isoch,
            2 => EndpointTransfer::Bulk,
            3 => EndpointTransfer::Interrupt,
            _ => unreachable!(),
        }
    }

    fn from_raw(n: u8) -> Option<Self> {
        matches!(n >> 4 & 0x3, 0 | 1 | 2).then(|| Self(n))
    }
}

impl fmt::Debug for EndpointAttributes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(stringify!(EndpointAttributes))
            .field("usage", &self.usage())
            .field("sync", &self.sync())
            .field("transfer", &self.transfer())
            .finish()
    }
}

#[derive(Debug)]
pub enum EndpointUsage {
    Data,
    Feedback,
    Implicit,
}

#[derive(Debug)]
pub enum EndpointSync {
    None,
    Async,
    Adapt,
    Sync,
}

#[derive(Debug)]
pub enum EndpointTransfer {
    Control,
    Isoch,
    Bulk,
    Interrupt,
}

#[derive(Debug)]
pub enum Direction {
    In,
    Out,
}

#[derive(Debug)]
pub enum InvalidEndpoint {
    UnexpectedLength,
    InvalidAddress,
    InvalidAttributes,
}
