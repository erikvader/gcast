use std::{os::raw::c_ulong, sync::Arc};

#[derive(thiserror::Error, Debug)]
pub enum SafeLibMpvError {
    Loadfiles {
        index: usize,
        error: Arc<SafeLibMpvError>,
    },
    VersionMismatch {
        linked: c_ulong,
        loaded: c_ulong,
    },
    InvalidUtf8,
    Null,
    Raw(libmpv::MpvError),
}

impl std::fmt::Display for SafeLibMpvError {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}

impl From<libmpv::Error> for SafeLibMpvError {
    fn from(mpv: libmpv::Error) -> Self {
        use libmpv::Error::*;
        match mpv {
            Loadfiles { index, error } => Self::Loadfiles {
                index,
                error: Arc::new((*error).clone().into()), // TODO: unwrap_or_clone when stable
            },
            VersionMismatch { linked, loaded } => {
                Self::VersionMismatch { linked, loaded }
            }
            InvalidUtf8 => Self::InvalidUtf8,
            Null => Self::Null,
            Raw(me) => Self::Raw(me),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum MpvError {
    #[error("mpv error: {0}")]
    Mpv(#[from] SafeLibMpvError),
    #[error("Mpv::Exited: thread is not running anymore")]
    Exited,
}

impl From<libmpv::Error> for MpvError {
    fn from(mpv: libmpv::Error) -> Self {
        let safe: SafeLibMpvError = mpv.into();
        safe.into()
    }
}
