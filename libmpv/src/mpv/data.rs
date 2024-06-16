use crate::mpv::macros::enum_int_map;
use crate::{bindings::*, see_string::SeeString};
use std::ffi::CString;
use std::ptr::{self, addr_of_mut};
use std::{collections::HashMap, ffi::CStr};

enum_int_map! {pub Format (mpv_format) {
    (None, MPV_FORMAT_NONE),
    (String, MPV_FORMAT_STRING),
    (OsdString, MPV_FORMAT_OSD_STRING),
    (Flag, MPV_FORMAT_FLAG),
    (Int64, MPV_FORMAT_INT64),
    (Double, MPV_FORMAT_DOUBLE),
    (Node, MPV_FORMAT_NODE),
    (NodeArray, MPV_FORMAT_NODE_ARRAY),
    (NodeMap, MPV_FORMAT_NODE_MAP),
    (ByteArray, MPV_FORMAT_BYTE_ARRAY),
}}

#[derive(Clone, Debug)]
pub enum Node {
    String(String),
    Flag(bool),
    Int64(i64),
    Double(f64),
    Array(Vec<Node>),
    Map(HashMap<String, Node>),
    // NOTE: ignoring byte array
    None,
    Unknown(Format),
}

impl Node {
    pub fn try_to_string(&self) -> Option<&str> {
        if let Node::String(s) = self {
            Some(s)
        } else {
            None
        }
    }

    pub fn try_to_flag(&self) -> Option<bool> {
        if let Node::Flag(s) = self {
            Some(*s)
        } else {
            None
        }
    }

    pub fn try_to_i64(&self) -> Option<i64> {
        if let Node::Int64(s) = self {
            Some(*s)
        } else {
            None
        }
    }

    pub fn try_to_double(&self) -> Option<f64> {
        if let Node::Double(s) = self {
            Some(*s)
        } else {
            None
        }
    }
}

unsafe fn ref_to_node(ptr: &mpv_node) -> Node {
    let u = ptr.u;
    match Format::from(ptr.format) {
        Format::None => Node::None,
        Format::String => Node::String(String::from_mpv_data(&u.string.cast_const())),
        Format::Flag => Node::Flag(u.flag != 0),
        Format::Int64 => Node::Int64(u.int64),
        Format::Double => Node::Double(u.double),
        Format::NodeArray => {
            let num = (*u.list).num as isize;
            let values = (*u.list).values;
            assert!(num == 0 || !values.is_null());
            let array = (0..num)
                .into_iter()
                .map(|i| ref_to_node(&*values.offset(i)))
                .collect();
            Node::Array(array)
        }
        Format::NodeMap => {
            let num = (*u.list).num as isize;
            let values = (*u.list).values;
            assert!(num == 0 || !values.is_null());
            let keys = (*u.list).keys;
            assert!(num == 0 || !keys.is_null());
            let map = (0..num)
                .into_iter()
                .map(|i| {
                    (
                        String::from_mpv_data(&(*keys.offset(i)).cast_const()),
                        ref_to_node(&*values.offset(i)),
                    )
                })
                .collect();
            Node::Map(map)
        }
        format => Node::Unknown(format),
    }
}

pub(crate) trait ToMpvData {
    type Output;

    fn to_mpv_data(&self) -> Self::Output;
}

impl<'a> ToMpvData for SeeString<'a> {
    type Output = *const libc::c_char;

    fn to_mpv_data(&self) -> Self::Output {
        self.as_cstr().as_ptr()
    }
}

impl ToMpvData for i64 {
    type Output = int64_t;

    fn to_mpv_data(&self) -> Self::Output {
        *self
    }
}

impl ToMpvData for f64 {
    type Output = libc::c_double;

    fn to_mpv_data(&self) -> Self::Output {
        *self
    }
}

impl ToMpvData for bool {
    type Output = libc::c_int;

    fn to_mpv_data(&self) -> Self::Output {
        (*self).then_some(1).unwrap_or(0)
    }
}

pub(crate) trait FromMpvData {
    type Input;

    fn from_mpv_data(mpv: &Self::Input) -> Self;
}

impl FromMpvData for CString {
    type Input = *const libc::c_char;

    fn from_mpv_data(mpv: &Self::Input) -> Self {
        assert!(!mpv.is_null());
        unsafe { CStr::from_ptr(*mpv) }.into()
    }
}

/// NOTE: The returned property should be UTF-8 except for a few things, see the header
/// file. From the doc of MPV_FORMAT_STRING: although the encoding is usually UTF-8, this
/// is not always the case. File tags often store strings in some legacy codepage, and
/// even filenames don't necessarily have to be in UTF-8 (at least on Linux).
impl FromMpvData for String {
    type Input = *const libc::c_char;

    fn from_mpv_data(mpv: &Self::Input) -> Self {
        let cstr = CString::from_mpv_data(mpv);
        cstr.to_string_lossy().into_owned()
    }
}

impl FromMpvData for Node {
    type Input = mpv_node;

    fn from_mpv_data(mpv: &Self::Input) -> Self {
        unsafe { ref_to_node(mpv) }
    }
}

impl FromMpvData for bool {
    type Input = libc::c_int;

    fn from_mpv_data(mpv: &Self::Input) -> Self {
        *mpv != 0
    }
}

impl FromMpvData for f64 {
    type Input = libc::c_double;

    fn from_mpv_data(mpv: &Self::Input) -> Self {
        *mpv
    }
}

impl FromMpvData for i64 {
    type Input = int64_t;

    fn from_mpv_data(mpv: &Self::Input) -> Self {
        *mpv
    }
}

pub(crate) trait AllocMpvData {
    fn free(self);
    fn empty() -> Self;
}

impl AllocMpvData for *const libc::c_char {
    fn free(self) {
        assert!(!self.is_null());
        unsafe { mpv_free(self.cast_mut().cast()) };
    }

    fn empty() -> Self {
        ptr::null()
    }
}

impl AllocMpvData for mpv_node {
    fn free(mut self) {
        unsafe { mpv_free_node_contents(addr_of_mut!(self)) };
    }

    fn empty() -> Self {
        mpv_node {
            u: mpv_node_u { int64: 0 },
            format: Format::None.to_int(),
        }
    }
}

impl AllocMpvData for libc::c_int {
    fn free(self) {}

    fn empty() -> Self {
        0
    }
}

impl AllocMpvData for libc::c_double {
    fn free(self) {}

    fn empty() -> Self {
        0.0
    }
}

impl AllocMpvData for int64_t {
    fn free(self) {}

    fn empty() -> Self {
        0
    }
}
