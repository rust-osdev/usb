use core::fmt;

#[derive(Debug)]
pub struct Configuration {
    pub total_length: u16,
    pub num_interfaces: u8,
    pub configuration_value: u8,
    /// Value which when used as an argument in the SET_CONFIGURATION request,
    /// causes the device to assume the configuration described by this descriptor.
    pub index_configuration: u8,
    pub attributes: ConfigurationAttributes,
    pub max_power: u8,
}

impl Configuration {
    pub(crate) fn from_raw(buf: &[u8]) -> Result<Self, InvalidConfiguration> {
        if let &[a, b, c, d, e, f, g] = buf {
            Ok(Configuration {
                total_length: u16::from_le_bytes([a, b]),
                num_interfaces: c,
                configuration_value: d,
                index_configuration: e,
                attributes: ConfigurationAttributes(f),
                max_power: g,
            })
        } else {
            Err(InvalidConfiguration::UnexpectedLength)
        }
    }
}

pub struct ConfigurationAttributes(u8);

macro_rules! flag {
    ($i:literal $f:ident) => {
        fn $f(&self) -> bool {
            self.0 & 1 << $i != 0
        }
    };
}

impl ConfigurationAttributes {
    flag!(6 self_powered);
    flag!(5 remote_wakeup);
}

impl fmt::Debug for ConfigurationAttributes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut f = f.debug_set();
        self.self_powered()
            .then(|| f.entry(&format_args!("SELF_POWERED")));
        self.remote_wakeup()
            .then(|| f.entry(&format_args!("REMOTE_WAKEUP")));
        f.finish()
    }
}

#[derive(Debug)]
pub enum InvalidConfiguration {
    UnexpectedLength,
}
