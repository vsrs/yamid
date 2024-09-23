use uuid::Uuid;

pub mod error;

pub type Result<T> = std::result::Result<T, error::Error>;

#[derive(PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MachineId(Uuid);

impl AsRef<Uuid> for MachineId {
    #[inline]
    fn as_ref(&self) -> &Uuid {
        &self.0
    }
}

impl AsRef<[u8]> for MachineId {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl MachineId {
    #[cfg(windows)]
    pub fn new() -> Result<Self> {
        use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

        let guid_str = RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey("SOFTWARE\\Microsoft\\Cryptography")
            .and_then(|key| key.get_value::<String, _>("MachineGuid"))?;

        let machine_uuid = Uuid::parse_str(&guid_str)?;

        Ok(Self(machine_uuid))
    }

    #[cfg(target_os = "linux")]
    pub fn new() -> Result<Self> {
        use std::fs::read_to_string;

        let guid_str = read_to_string("/var/lib/dbus/machine-id")
            .or_else(|_| read_to_string("/etc/machine-id"))?;
        let machine_uuid = Uuid::parse_str(guid_str.trim_end())?;

        Ok(Self(machine_uuid))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_machine_id() {
        let first = MachineId::new().unwrap();
        let second = MachineId::new().unwrap();

        assert_eq!(first, second);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde() {
        let id = MachineId::new().unwrap();
        let s = serde_json::to_string(&id).unwrap();

        let de: MachineId = serde_json::from_str(&s).unwrap();
        assert_eq!(id, de);
    }
}
