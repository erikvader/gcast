use super::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct SendStatus;

into_ToServer!(SendStatus);
