use super::*;

macro_rules! into_Status {
    ($msg:ident) => {
        impl From<$msg> for MessageKind {
            fn from(m: $msg) -> MessageKind {
                Status::$msg(m).into()
            }
        }
    };
}

pub mod mpv;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum Status {
    None,
    Spotify,
    Mpv(mpv::Mpv),
}

into_ToClient!(Status);
