use crate::Error;
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct MacAddress([u8; 6]);

impl MacAddress {
    pub fn parse(s: &str) -> Result<Self, Error> {
        let mut result = [0; 6];

        for (i, byte) in s.split(':').enumerate() {
            if byte.len() != 2 {
                return Err(Error::InvalidMacAddress);
            }

            let byte = u8::from_str_radix(byte, 16).map_err(|_| Error::InvalidMacAddress)?;

            result[i] = byte;
        }

        Ok(Self(result))
    }

    pub fn send_magic_packet(&self) -> Result<(), Error> {
        wake_on_lan::MagicPacket::new(&self.0)
            .send()
            .map_err(Error::WakeOnLan)
    }
}

impl Display for MacAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, byte) in self.0.iter().enumerate() {
            if i != 0 {
                write!(f, ":")?;
            }

            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}
