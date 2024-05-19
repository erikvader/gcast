use std::{
    borrow::Cow,
    ffi::{CStr, CString, OsStr, OsString},
    path::{Path, PathBuf},
};

#[derive(Debug, PartialEq, Eq)]
pub struct SeeString<'a> {
    inner: Cow<'a, CStr>,
}

impl SeeString<'_> {
    pub fn into_cstring(self) -> CString {
        self.inner.into()
    }

    pub fn as_cstr(&self) -> &CStr {
        self.as_ref()
    }
}

impl AsRef<CStr> for SeeString<'_> {
    fn as_ref(&self) -> &CStr {
        &self.inner
    }
}

impl std::ops::Deref for SeeString<'_> {
    type Target = CStr;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<CString> for SeeString<'_> {
    fn from(value: CString) -> Self {
        Self {
            inner: value.into(),
        }
    }
}

impl<'a> From<&'a CStr> for SeeString<'a> {
    fn from(value: &'a CStr) -> Self {
        Self {
            inner: value.into(),
        }
    }
}

impl From<&str> for SeeString<'_> {
    fn from(value: &str) -> Self {
        value.to_owned().into()
    }
}

impl From<&OsStr> for SeeString<'_> {
    fn from(value: &OsStr) -> Self {
        // TODO: check if it ends with a null byte and use directly if it does?
        value.to_owned().into()
    }
}

impl From<String> for SeeString<'_> {
    fn from(value: String) -> Self {
        let cstring = CString::new(value)
            .expect("rust strings can have nulls, but hope it doesn't");
        cstring.into()
    }
}

#[cfg(unix)]
impl From<OsString> for SeeString<'_> {
    fn from(value: OsString) -> Self {
        use std::os::unix::ffi::OsStringExt;
        let cstring = CString::new(value.into_vec())
            .expect("let's hope it doesn't contain any nulls");
        cstring.into()
    }
}

#[cfg(unix)]
impl From<PathBuf> for SeeString<'_> {
    fn from(value: PathBuf) -> Self {
        value.into_os_string().into()
    }
}

#[cfg(unix)]
impl From<&Path> for SeeString<'_> {
    fn from(value: &Path) -> Self {
        value.to_owned().into()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn cstring() {
        let cstring: CString = c"hej".to_owned();
        let see1: SeeString<'_> = cstring.into();

        let cstr: &CStr = c"hej";
        let see2: SeeString<'_> = cstr.into();
        assert_eq!(see1, see2);
    }

    #[test]
    fn string() {
        let string: String = "hej".to_owned();
        let see1: SeeString<'_> = string.into();

        let sstr: &str = "hej";
        let see2: SeeString<'_> = sstr.into();
        assert_eq!(see1, see2);
    }
}
