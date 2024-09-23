A lightweight crate for retrieving the unique machine ID without needing root/admin privileges.

# IDs sources
- Windows: the `MachineGuid` value from `HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Cryptography` ([Identifying Unique Windows Installation](https://learn.microsoft.com/en-us/answers/questions/1489139/identifying-unique-windows-installation))
- Linux: `/etc/machine-id` with a fallback to `/var/lib/dbus/machine-id` ([man page](https://man7.org/linux/man-pages/man5/machine-id.5.html))
---
- BSD /etc/hostid and smbios.system.uuid as a fallback
- macOS and OSX `IOPlatformExpertDevice`?
- Android

# Security Considerations
A machine ID uniquely identifies the host and should be treated as confidential, avoiding exposure in untrusted environments.
If your application requires a stable unique identifier, avoid using the machine as it is.
Instead, hash the machine ID securely with a fixed, application-specific [salt](https://en.wikipedia.org/wiki/Salt_(cryptography)).

> [!WARNING]  
> Hashing IDs is not only a best practice today, if you store the ID somewhere (like your remote server), you actually must do it by law according to [GDPR (see identifiers)](https://gdpr.eu/eu-gdpr-personal-data/) and similar regulations.

> [!TIP]  
> Virtual machines deployed from the same template often share the same machine ID. To differentiate them, include the MAC address when hashing.
