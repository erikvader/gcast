macro_rules! enum_int_map {
    ($name:ident ($type:ty) {$(($r:ident, $c:ident)),* $(,)*}) => {
        #[derive(Debug, Copy, Clone)]
        // TODO: support specifying the publicness and remove the private modules
        pub enum $name {
            $($r),*,
            Unknown($type),
        }

        impl $name {
            pub const fn from_int(int: $type) -> Self {
                match () {
                    $(_ if int == $c => Self::$r),*,
                    _ => Self::Unknown(int),
                }
            }

            pub const fn to_int(self) -> $type {
                match self {
                    $(Self::$r => $c),*,
                    Self::Unknown(int) => int,
                }
            }

            pub const fn is_unknown(self) -> bool {
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
    ($name:ident {$(($r:ident, $c:literal)),* $(,)*}) => {
        #[derive(Debug, Copy, Clone)]
        pub enum $name {
            $($r),*,
            Unknown,
        }

        impl $name {
            pub fn from_cstr(cstr: &std::ffi::CStr) -> Self {
                match () {
                    $(_ if cstr == $c => Self::$r),*,
                    _ => Self::Unknown,
                }
            }

            pub fn from_ptr(ptr: *const libc::c_char) -> Self {
                assert!(!ptr.is_null());
                Self::from_cstr(unsafe{std::ffi::CStr::from_ptr(ptr)})
            }

            pub const fn as_cstr(self) -> &'static std::ffi::CStr {
                match self {
                    $(Self::$r => $c),*,
                    Self::Unknown => c"<UNKNOWN>",
                }
            }

            pub const fn as_ptr(self) -> *const libc::c_char {
                self.as_cstr().as_ptr()
            }

            pub const fn is_unknown(self) -> bool {
                matches!(self, Self::Unknown)
            }
        }

        impl From<&std::ffi::CStr> for $name {
            fn from(int: &std::ffi::CStr) -> Self {
                Self::from_cstr(int)
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
        match int {
            0.. => Ok(int),
            _ => Err($crate::mpv::error::Error::ErrorCode(
                $crate::mpv::error::ErrorCode::from_int(int),
            )),
        }
    }};
}
pub(crate) use mpv_try;

macro_rules! mpv_try_null {
    ($expr:expr) => {{
        let ptr = ($expr);
        if ptr.is_null() {
            return Err($crate::mpv::error::Error::NullPtr);
        }
        Ok(ptr)
    }};
}
pub(crate) use mpv_try_null;

macro_rules! mpv_try_unknown {
    ($expr:expr) => {{
        let val = ($expr);
        if val.is_unknown() {
            return Err($crate::mpv::error::Error::Unknown);
        }
        Ok(val)
    }};
}
pub(crate) use mpv_try_unknown;
