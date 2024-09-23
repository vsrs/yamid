use uuid::Uuid;

pub mod error;

pub type Result<T> = std::result::Result<T, error::Error>;

#[derive(PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MachineId(Uuid);

impl std::fmt::Display for MachineId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::UpperHex::fmt(&self.0, f)
    }
}

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

        let guid_str = read_to_string("/etc/machine-id")
            .and_then(|data| {
                if data.is_empty() {
                    Err(std::io::Error::new(std::io::ErrorKind::InvalidData, ""))
                } else {
                    Ok(data)
                }
            })
            .or_else(|_| read_to_string("/var/lib/dbus/machine-id"))?;
        let machine_uuid = Uuid::parse_str(guid_str.trim_end())?;

        Ok(Self(machine_uuid))
    }

    #[cfg(target_os = "macos")]
    pub fn new() -> Result<Self> {
        use apple_sys::IOKit as io;
        use core_foundation::{
            base::TCFType,
            string::{CFString, CFStringRef},
        };

        struct ObjectReleaser(u32);
        impl Drop for ObjectReleaser {
            fn drop(&mut self) {
                unsafe { io::IOObjectRelease(self.0) };
            }
        }

        let uuid_str = unsafe {
            let root = io::IORegistryEntryFromPath(
                io::kIOMasterPortDefault,
                "IOService:/\0".as_ptr() as _,
            );

            if root == io::MACH_PORT_NULL {
                return Err(std::io::Error::last_os_error().into());
            }

            let root = ObjectReleaser(root);
            let key = CFString::from_static_string("IOPlatformUUID");
            let uuid_cref: CFStringRef = io::IORegistryEntryCreateCFProperty(
                root.0,
                key.as_CFTypeRef() as _,
                io::kCFAllocatorDefault,
                0,
            ) as _;

            if uuid_cref.is_null() {
                return Err(std::io::Error::last_os_error().into());
            }
            CFString::wrap_under_create_rule(uuid_cref).to_string()
        };

        Ok(Self(uuid::Uuid::parse_str(&uuid_str)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_machine_id() {
        let first = MachineId::new().unwrap();
        let second = MachineId::new().unwrap();

        println!("Machine id: {first}");

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
