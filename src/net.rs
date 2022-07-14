use serde::{Deserialize, Serialize};

// TODO: inte använda bincode för just denna?
#[derive(Debug, Serialize, Deserialize)]
struct Header {
    size: u16,
    size_inv: u16,
}

impl Header {
    const MAX_SIZE: usize = u16::MAX as usize;

    pub fn new(size: u16) -> Self {
        Self {
            size,
            size_inv: size ^ u16::MAX,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.size == (self.size_inv ^ u16::MAX)
    }

    pub fn size(&self) -> u16 {
        self.size
    }
}
