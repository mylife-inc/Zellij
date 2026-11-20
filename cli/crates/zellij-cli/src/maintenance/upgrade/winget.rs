//! WinGet portable ownership, including custom locations and launcher links.
//! Registry contract: microsoft/winget-cli v1.29.380 PortableARPEntry.cpp and
//! PortableFlow.cpp. Never infer ownership just from a directory name.
use std::{io, path::Path};

#[cfg(windows)]
const PACKAGE_ID: &str = "Shebka.Zel";

fn registered_file_is_current(executable: &Path, registered: &Path) -> io::Result<bool> {
    if !registered.is_absolute() {
        return Ok(false);
    }
    match same_file::is_same_file(executable, registered) {
        Ok(same) => Ok(same),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

#[cfg(windows)]
pub(super) fn owns(executable: &Path) -> io::Result<bool> {
    use windows_sys::Win32::System::Registry::*;
    // WinGet uses the shared HKCU view for user installations. Machine ARM64
    // portable entries use the 32-bit uninstall view in the pinned contract.
    for (hive, subkey, view) in [
        (
            HKEY_CURRENT_USER,
            r"Software\Microsoft\Windows\CurrentVersion\Uninstall",
            KEY_WOW64_64KEY,
        ),
        (
            HKEY_LOCAL_MACHINE,
            r"Software\Microsoft\Windows\CurrentVersion\Uninstall",
            KEY_WOW64_64KEY,
        ),
        (
            HKEY_LOCAL_MACHINE,
            r"Software\Wow6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
            KEY_WOW64_32KEY,
        ),
    ] {
        if let Some(root) = registry::Key::open(hive, subkey.as_ref(), view)?
            && registry::owns_in(&root, executable)?
        {
            return Ok(true);
        }
    }
    Ok(false)
}

#[cfg(windows)]
mod registry {
    use super::*;
    use std::{
        ffi::{OsStr, OsString},
        os::windows::ffi::{OsStrExt, OsStringExt},
        path::PathBuf,
        ptr,
    };
    use windows_sys::Win32::{
        Foundation::{ERROR_FILE_NOT_FOUND, ERROR_NO_MORE_ITEMS, ERROR_SUCCESS},
        System::Registry::*,
    };

    pub(super) struct Key(HKEY);
    impl Drop for Key {
        fn drop(&mut self) {
            unsafe {
                RegCloseKey(self.0);
            }
        }
    }
    fn wide(value: &OsStr) -> Vec<u16> {
        value.encode_wide().chain(Some(0)).collect()
    }
    fn error(code: u32) -> io::Error {
        io::Error::from_raw_os_error(code as i32)
    }
    impl Key {
        pub(super) fn open(parent: HKEY, name: &OsStr, view: u32) -> io::Result<Option<Self>> {
            let mut key = ptr::null_mut();
            let status =
                unsafe { RegOpenKeyExW(parent, wide(name).as_ptr(), 0, KEY_READ | view, &mut key) };
            match status {
                ERROR_SUCCESS => Ok(Some(Self(key))),
                ERROR_FILE_NOT_FOUND => Ok(None),
                code => Err(error(code)),
            }
        }
        fn string(&self, name: &str) -> io::Result<Option<OsString>> {
            // Bound registry data, including concurrent size changes. WinGet
            // writes these values as REG_SZ; other types are not ownership.
            let mut data = vec![0u16; 32768];
            let mut bytes = (data.len() * 2) as u32;
            let mut kind = 0;
            let status = unsafe {
                RegQueryValueExW(
                    self.0,
                    wide(name.as_ref()).as_ptr(),
                    ptr::null(),
                    &mut kind,
                    data.as_mut_ptr().cast(),
                    &mut bytes,
                )
            };
            match status {
                ERROR_FILE_NOT_FOUND => return Ok(None),
                ERROR_SUCCESS => {}
                code => return Err(error(code)),
            }
            if kind != REG_SZ || bytes == 0 || bytes % 2 != 0 {
                return Ok(None);
            }
            data.truncate((bytes / 2) as usize);
            if data.last() == Some(&0) {
                data.pop();
            }
            if data.contains(&0) {
                return Ok(None);
            }
            Ok(Some(OsString::from_wide(&data)))
        }
    }
    pub(super) fn owns_in(root: &Key, executable: &Path) -> io::Result<bool> {
        let prefix = format!("{PACKAGE_ID}_");
        for index in 0.. {
            let mut name = [0u16; 256];
            let mut length = name.len() as u32;
            let status = unsafe {
                RegEnumKeyExW(
                    root.0,
                    index,
                    name.as_mut_ptr(),
                    &mut length,
                    ptr::null(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                )
            };
            match status {
                ERROR_NO_MORE_ITEMS => return Ok(false),
                ERROR_SUCCESS => {}
                code => return Err(error(code)),
            }
            let name = OsString::from_wide(&name[..length as usize]);
            // Product codes are generated as package ID + '_' + source ID.
            // Avoid inspecting unrelated applications' values/permissions.
            if !name
                .to_string_lossy()
                .to_ascii_lowercase()
                .starts_with(&prefix.to_ascii_lowercase())
            {
                continue;
            }
            let Some(key) = Key::open(root.0, &name, 0)? else {
                continue;
            };
            if key.string("WinGetPackageIdentifier")?.as_deref() != Some(OsStr::new(PACKAGE_ID))
                || key.string("WinGetInstallerType")?.as_deref() != Some(OsStr::new("portable"))
            {
                continue;
            }
            // Single EXE packages record TargetFullPath. ZIP packages record
            // their directory and keep a separate WinGet file index. Our ZIP
            // contract has exactly one root executable named zel.exe.
            if let Some(target) = key.string("TargetFullPath")?
                && registered_file_is_current(executable, Path::new(&target))?
            {
                return Ok(true);
            }
            if let Some(location) = key.string("InstallLocation")?
                && registered_file_is_current(executable, &PathBuf::from(location).join("zel.exe"))?
            {
                return Ok(true);
            }
        }
        unreachable!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn custom_location_and_hardlink_are_owned_but_a_standalone_copy_is_not() {
        let temp = tempfile::tempdir().unwrap();
        let managed = temp.path().join("custom 日本");
        fs::create_dir(&managed).unwrap();
        let installed = managed.join("zel.exe");
        fs::write(&installed, b"binary").unwrap();
        let alias = temp.path().join("alias.exe");
        fs::hard_link(&installed, &alias).unwrap();
        let standalone = temp.path().join("standalone.exe");
        fs::copy(&installed, &standalone).unwrap();
        assert!(registered_file_is_current(&installed, &installed).unwrap());
        assert!(registered_file_is_current(&alias, &installed).unwrap());
        assert!(!registered_file_is_current(&standalone, &installed).unwrap());
        assert!(!registered_file_is_current(&installed, Path::new("zel.exe")).unwrap());
        assert!(!registered_file_is_current(&installed, &temp.path().join("missing.exe")).unwrap());
    }
}
