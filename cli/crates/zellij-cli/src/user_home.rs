//! Persistent per-user resources, independent of the executable's installer.
use std::{ffi::OsString, path::PathBuf};
use zellij_core::{Error, Result};

pub(crate) fn root() -> Result<PathBuf> {
    resolve(std::env::var_os("ZELLIJ_HOME"), std::env::home_dir())
}

fn resolve(configured: Option<OsString>, user_home: Option<PathBuf>) -> Result<PathBuf> {
    let root = match configured.filter(|value| !value.is_empty()) {
        Some(value) => PathBuf::from(value),
        None => user_home
            .filter(|path| !path.as_os_str().is_empty())
            .ok_or_else(|| {
                Error::InvalidArgument(
                    "Cannot locate your home directory; set ZELLIJ_HOME to an absolute path".into(),
                )
            })?
            .join("Dev")
            .join("config")
            .join("zellij"),
    };
    if !root.is_absolute() {
        return Err(Error::InvalidArgument(
            "ZELLIJ_HOME must be an absolute path (expand ~ to your home directory)".into(),
        ));
    }
    Ok(root)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn home_policy_is_independent_of_installer_and_platform_cache_directories() {
        let temp = tempfile::tempdir().unwrap();
        let home = temp.path().join("User 日本");
        let expected = home.join("Dev").join("config").join("zellij");
        assert_eq!(resolve(None, Some(home.clone())).unwrap(), expected);
        assert_eq!(
            resolve(Some(OsString::new()), Some(home)).unwrap(),
            expected
        );
        let custom = temp.path().join("Zellij resources café");
        assert_eq!(
            resolve(Some(custom.clone().into_os_string()), None).unwrap(),
            custom
        );
        assert!(resolve(None, None).is_err());
        for path in ["relative", "~/zellij"] {
            assert!(resolve(Some(path.into()), Some(temp.path().to_owned())).is_err());
        }
    }
}
