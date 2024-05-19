macro_rules! enum_int_map {
    ($vis:vis $name:ident ($type:ty) {$(($r:ident, $c:ident)),* $(,)*}) => {
        #[derive(Debug, Copy, Clone)]
        $vis enum $name {
            $($r),*,
            Unknown($type),
        }

        #[allow(dead_code)]
        impl $name {
            $vis const fn from_int(int: $type) -> Self {
                match () {
                    $(_ if int == $c => Self::$r),*,
                    _ => Self::Unknown(int),
                }
            }

            $vis const fn to_int(self) -> $type {
                match self {
                    $(Self::$r => $c),*,
                    Self::Unknown(int) => int,
                }
            }

            $vis const fn is_unknown(self) -> bool {
                matches!(self, Self::Unknown(_))
            }
        }

        impl From<$type> for $name {
            fn from(int: $type) -> Self {
                Self::from_int(int)
            }
        }

        impl From<$name> for $type {
            fn from(e: $name) -> Self {
                e.to_int()
            }
        }
    };
}
pub(crate) use enum_int_map;

macro_rules! enum_cstr_map {
    ($vis:vis $name:ident {$(($r:ident, $c:literal)),* $(,)*}) => {
        #[derive(Debug, Clone)]
        $vis enum $name {
            $($r),*,
            Unknown(#[allow(dead_code)] std::ffi::CString),
        }

        #[allow(dead_code)]
        impl $name {
            $vis fn from_str<'a>(cstr: impl Into<SeeString<'a>>) -> Self {
                let cstr = cstr.into();
                match () {
                    $(_ if cstr.as_ref() == $c => Self::$r),*,
                    _ => Self::Unknown(cstr.into_cstring()),
                }
            }

            $vis fn from_ptr(ptr: *const libc::c_char) -> Self {
                assert!(!ptr.is_null());
                Self::from_str(unsafe{std::ffi::CStr::from_ptr(ptr)})
            }

            $vis const fn as_cstr(&self) -> &'static std::ffi::CStr {
                match self {
                    $(Self::$r => $c),*,
                    Self::Unknown(_) => c"<UNKNOWN>",
                }
            }

            $vis fn as_str(&self) -> &'static str {
                self.as_cstr().to_str().unwrap()
            }

            $vis const fn as_ptr(&self) -> *const libc::c_char {
                self.as_cstr().as_ptr()
            }

            $vis const fn is_unknown(&self) -> bool {
                matches!(self, Self::Unknown(_))
            }
        }

        impl From<&std::ffi::CStr> for $name {
            fn from(int: &std::ffi::CStr) -> Self {
                Self::from_str(int)
            }
        }

        impl From<$name> for &'static std::ffi::CStr {
            fn from(e: $name) -> Self {
                e.as_cstr()
            }
        }

        impl AsRef<std::ffi::CStr> for $name {
            fn as_ref(&self) -> &'static std::ffi::CStr {
                self.as_cstr()
            }
        }
    };
}
pub(crate) use enum_cstr_map;

macro_rules! mpv_try {
    ($expr:expr) => {{
        let int = ($expr);
        $crate::mpv::error::error_code(int)
            .map_err(|err| $crate::mpv::error::Error::ErrorCode(err))
    }};
}
pub(crate) use mpv_try;

macro_rules! mpv_try_null {
    ($expr:expr) => {{
        let ptr = ($expr);
        if ptr.is_null() {
            Err($crate::mpv::error::Error::NullPtr)
        } else {
            Ok(ptr)
        }
    }};
}
pub(crate) use mpv_try_null;

macro_rules! mpv_try_unknown {
    ($expr:expr) => {{
        let val = ($expr);
        if val.is_unknown() {
            Err($crate::mpv::error::Error::Unknown)
        } else {
            Ok(val)
        }
    }};
}
pub(crate) use mpv_try_unknown;
