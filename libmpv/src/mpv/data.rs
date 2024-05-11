use crate::bindings::*;
use crate::mpv::macros::enum_int_map;
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

/// NOTE: The returned property should be UTF-8 except for a few things, see the header
/// file. From the doc of MPV_FORMAT_STRING: although the encoding is usually UTF-8, this
/// is not always the case. File tags often store strings in some legacy codepage, and
/// even filenames don't necessarily have to be in UTF-8 (at least on Linux).
pub(crate) fn cstr_to_string(ptr: &CStr) -> String {
    ptr.to_string_lossy().into_owned()
}

pub(crate) unsafe fn ptr_to_string(ptr: *const libc::c_char) -> String {
    assert!(!ptr.is_null());
    cstr_to_string(CStr::from_ptr(ptr))
}

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

pub(crate) unsafe fn ptr_to_node(ptr: *const mpv_node) -> Node {
    assert!(!ptr.is_null());
    let u = (*ptr).u;
    match Format::from((*ptr).format) {
        Format::None => Node::None,
        Format::String => Node::String(ptr_to_string(u.string)),
        Format::Flag => Node::Flag(u.flag != 0),
        Format::Int64 => Node::Int64(u.int64),
        Format::Double => Node::Double(u.double),
        Format::NodeArray => {
            let num = (*u.list).num as isize;
            let values = (*u.list).values;
            assert!(num == 0 || !values.is_null());
            let array = (0..num)
                .into_iter()
                .map(|i| ptr_to_node(values.offset(i)))
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
                        ptr_to_string(*keys.offset(i)),
                        ptr_to_node(values.offset(i)),
                    )
                })
                .collect();
            Node::Map(map)
        }
        format => Node::Unknown(format),
    }
}
