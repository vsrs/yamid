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

impl From<MachineId> for Uuid {
    #[inline]
    fn from(machine_id: MachineId) -> Self {
        machine_id.0
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

    #[cfg(all(unix, not(target_os = "linux"), not(target_os = "macos")))]
    pub fn new() -> Result<Self> {
        let id = unix::host_uuid().or_else(|_| std::fs::read_to_string("/etc/hostid"))?;
        let machine_uuid = Uuid::parse_str(id.trim_end())?;
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

#[cfg(all(unix, not(target_os = "linux"), not(target_os = "macos")))]
mod unix {
    pub fn host_uuid() -> std::io::Result<String> {
        const KERN_HOSTUUID: i32 = 0x24i32;
        let vec = sysctl([libc::CTL_KERN, KERN_HOSTUUID])?;

        Ok(vec
            .into_iter()
            .take_while(|ch| *ch != 0)
            .map(|ch| ch as char)
            .collect::<String>())
    }

    pub fn sysctl<const N: usize>(mib: [i32; N]) -> std::io::Result<Vec<u8>> {
        use std::ptr;
        let (mut m, mut n) = (mib, 0);
        let r = unsafe {
            libc::sysctl(
                m.as_mut_ptr() as _,
                m.len() as _,
                ptr::null_mut(),
                &mut n,
                ptr::null_mut(),
                0,
            )
        };

        if r != 0 {
            return Err(std::io::Error::from_raw_os_error(r));
        }
        let mut b = Vec::with_capacity(n);
        let s = unsafe {
            let res = libc::sysctl(
                m.as_mut_ptr() as _,
                m.len() as _,
                b.as_mut_ptr() as _,
                &mut n,
                ptr::null_mut(),
                0,
            );
            b.set_len(n);
            res
        };
        if s != 0 {
            Err(std::io::Error::from_raw_os_error(s))
        } else {
            Ok(b)
        }
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

    #[test]
    fn test_into_uuid() {
        let machine_id = MachineId::new().unwrap();
        let expected_uuid: Uuid = *machine_id.as_ref();
        let uuid: Uuid = machine_id.into();

        assert_eq!(expected_uuid, uuid);
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
