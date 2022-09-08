#[derive(Debug)]
pub struct Device {
    pub usb: u16,
    pub class: u8,
    pub subclass: u8,
    pub protocol: u8,
    pub max_packet_size_0: u8,
    pub vendor: u16,
    pub product: u16,
    pub device: u16,
    pub index_manufacturer: u8,
    pub index_product: u8,
    pub index_serial_number: u8,
    pub num_configurations: u8,
}

impl Device {
    pub(crate) fn from_raw(buf: &[u8]) -> Result<Self, InvalidDevice> {
        if buf.len() != 16 {
            return Err(InvalidDevice::UnexpectedLength);
        }
        let f1 = |i: usize| buf[i - 2];
        let f2 = |i: usize| u16::from_le_bytes(buf[i - 2..i].try_into().unwrap());
        Ok(Device {
            usb: f2(2),
            class: f1(4),
            subclass: f1(5),
            protocol: f1(6),
            max_packet_size_0: f1(7),
            vendor: f2(8),
            product: f2(10),
            device: f2(12),
            index_manufacturer: f1(14),
            index_product: f1(15),
            index_serial_number: f1(16),
            num_configurations: f1(17),
        })
    }
}

#[derive(Debug)]
pub enum InvalidDevice {
    UnexpectedLength,
}
