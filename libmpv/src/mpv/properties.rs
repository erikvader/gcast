use std::{ffi::CStr, ptr};

use crate::{
    bindings::*,
    mpv::{data::ptr_to_string, macros::mpv_try_null},
};

use self::private::*;
use super::{
    data::{private::Format, ptr_to_node, Node},
    macros::{enum_cstr_map, mpv_try, mpv_try_unknown},
    Handle, Init, Result, Uninit,
};

macro_rules! properties {
    (@inner () -> ($(($name: ident, $type:ty))*)) => {
        #[derive(Debug, Clone)]
        pub enum PropertyValue {
            $($name($type)),*
        }
    };
    (@inner ((Flag,
              $prop:ident,
              $(Get $getter:ident $(,)*)*
              $(Set $setter:ident $(,)*)*
              $(Obs $obs:ident $(,)*)*
    ) $($rest:tt)*) -> ($($arms:tt)*)) => {
        impl Handle<Init> {
            $(
                pub fn $getter(&mut self) -> Result<bool> {
                    self.get_property_flag(Property::$prop)
                }
            )*
            $(
                pub fn $setter(&mut self, value: bool) -> Result<()> {
                    self.set_property_flag(Property::$prop, value)
                }
            )*
            $(
                pub fn $obs(&mut self) -> Result<()> {
                    self.observe_property(Property::$prop, Format::Flag)
                }
            )*
        }
        properties!{@inner ($($rest)*) -> ($($arms)* ($prop, bool))}
    };
    (@inner ((Int64,
              $prop:ident,
              $(Get $getter:ident $(,)*)*
              $(Set $setter:ident $(,)*)*
              $(Obs $obs:ident $(,)*)*
    ) $($rest:tt)*) -> ($($arms:tt)*)) => {
        impl Handle<Init> {
            $(
                pub fn $getter(&mut self) -> Result<i64> {
                    self.get_property_int(Property::$prop)
                }
            )*
            $(
                pub fn $setter(&mut self, value: i64) -> Result<()> {
                    self.set_property_int(Property::$prop, value)
                }
            )*
            $(
                pub fn $obs(&mut self) -> Result<()> {
                    self.observe_property(Property::$prop, Format::Int64)
                }
            )*
        }
        properties!{@inner ($($rest)*) -> ($($arms)* ($prop, i64))}
    };
    (@inner ((Double,
              $prop:ident,
              $(Get $getter:ident $(,)*)*
              $(Set $setter:ident $(,)*)*
              $(Obs $obs:ident $(,)*)*
    ) $($rest:tt)*) -> ($($arms:tt)*)) => {
        impl Handle<Init> {
            $(
                pub fn $getter(&mut self) -> Result<f64> {
                    self.get_property_double(Property::$prop)
                }
            )*
            $(
                pub fn $setter(&mut self, value: f64) -> Result<()> {
                    self.set_property_double(Property::$prop, value)
                }
            )*
            $(
                pub fn $obs(&mut self) -> Result<()> {
                    self.observe_property(Property::$prop, Format::Double)
                }
            )*
        }
        properties!{@inner ($($rest)*) -> ($($arms)* ($prop, f64))}
    };
    (@inner ((String,
              $prop:ident,
              $(Get $getter:ident $(,)*)*
              $(Obs $obs:ident $(,)*)*
    ) $($rest:tt)*) -> ($($arms:tt)*)) => {
        impl Handle<Init> {
            $(
                pub fn $getter(&mut self) -> Result<String> {
                    self.get_property_string(Property::$prop)
                }
            )*
            $(
                pub fn $obs(&mut self) -> Result<()> {
                    self.observe_property(Property::$prop, Format::String)
                }
            )*
        }
        properties!{@inner ($($rest)*) -> ($($arms)* ($prop, String))}
    };
    (@inner ((Node,
              $prop:ident,
              $(Get $getter:ident $(,)*)*
              $(Obs $obs:ident $(,)*)*
    ) $($rest:tt)*) -> ($($arms:tt)*)) => {
        impl Handle<Init> {
            $(
                pub fn $getter(&mut self) -> Result<Node> {
                    self.get_property_node(Property::$prop)
                }
            )*
            $(
                pub fn $obs(&mut self) -> Result<()> {
                    self.observe_property(Property::$prop, Format::Node)
                }
            )*
        }
        properties!{@inner ($($rest)*) -> ($($arms)* ($prop, Node))}
    };
    ($($rest:tt),* $(,)*) => {
        properties!{@inner ($($rest)*) -> ()}
    };
}
properties! {
    (Flag, Pause, Get is_paused, Set set_paused, Obs observe_paused),
    (String, MpvVersion, Get version),
    (String, MediaTitle, Get media_title, Obs observe_media_title),
    (Double, PlaybackTime, Get playback_time, Obs observe_playback_time),
    (Double, Duration, Get duration, Obs observe_duration),
    (Double, Volume, Get volume, Obs observe_volume),
    (Int64, Chapters, Get chapters, Obs observe_chapters),
    (Int64, Chapter, Get chapter, Obs observe_chapter),
    (Node, TrackList, Get track_list, Obs observe_track_list),
}

pub(crate) mod private {
    use super::*;

    enum_cstr_map! {Property {
        (MpvVersion, c"mpv-version"),
        (AudioDriver, c"ao"),
        (Pause, c"pause"),
        (InputDefaultBindings, c"input-default-bindings"),
        (InputVoKeyboard, c"input-vo-keyboard"),
        (MediaTitle, c"media-title"),
        (PlaybackTime, c"playback-time"),
        (Duration, c"duration"),
        (Volume, c"volume"),
        (Chapters, c"chapters"),
        (Chapter, c"chapter"),
        (TrackList, c"track-list"),
    }}
}

impl<T: super::private::InitState> Handle<T> {
    fn set_property_string(
        &mut self,
        prop: Property,
        value: impl AsRef<CStr>,
    ) -> Result<()> {
        mpv_try_unknown!(prop)?;
        let value = value.as_ref();
        mpv_try! {unsafe { mpv_set_property_string(self.ctx, prop.as_cstr().as_ptr(), value.as_ptr()) }}?;
        Ok(())
    }

    fn get_property_string(&mut self, prop: Property) -> Result<String> {
        mpv_try_unknown!(prop)?;
        let retval =
            mpv_try_null! {unsafe { mpv_get_property_string(self.ctx, prop.as_ptr()) }}?;
        let rust_str = unsafe { ptr_to_string(retval) };
        assert_ne!(retval as *const u8, rust_str.as_ptr());
        unsafe { mpv_free(retval as *mut libc::c_void) };
        Ok(rust_str)
    }

    fn get_property_node(&mut self, prop: Property) -> Result<Node> {
        mpv_try_unknown!(prop)?;
        let mut node = mpv_node {
            u: mpv_node_u { int64: 0 },
            format: Format::None.to_int(),
        };
        mpv_try! {unsafe { mpv_get_property(
            self.ctx,
            prop.as_ptr(),
            Format::Node.to_int(),
            ptr::from_mut(&mut node) as *mut libc::c_void
        ) }}?;
        let rust_node = unsafe { ptr_to_node(ptr::from_ref(&node)) };
        unsafe { mpv_free_node_contents(ptr::from_mut(&mut node)) };
        Ok(rust_node)
    }

    fn get_property_flag(&mut self, prop: Property) -> Result<bool> {
        mpv_try_unknown!(prop)?;
        let mut flag: libc::c_int = 0;
        mpv_try!(unsafe {
            mpv_get_property(
                self.ctx,
                prop.as_ptr(),
                Format::Flag.to_int(),
                ptr::from_mut(&mut flag) as *mut libc::c_void,
            )
        })?;
        Ok(flag != 0)
    }

    fn set_property_flag(&mut self, prop: Property, flag: bool) -> Result<()> {
        mpv_try_unknown!(prop)?;
        let mut flag: libc::c_int = if flag { 1 } else { 0 };
        mpv_try!(unsafe {
            mpv_set_property(
                self.ctx,
                prop.as_ptr(),
                Format::Flag.to_int(),
                ptr::from_mut(&mut flag) as *mut libc::c_void,
            )
        })?;
        Ok(())
    }

    fn get_property_double(&mut self, prop: Property) -> Result<f64> {
        mpv_try_unknown!(prop)?;
        let mut double: libc::c_double = 0.0;
        mpv_try!(unsafe {
            mpv_get_property(
                self.ctx,
                prop.as_ptr(),
                Format::Double.to_int(),
                ptr::from_mut(&mut double) as *mut libc::c_void,
            )
        })?;
        Ok(double)
    }

    #[allow(dead_code)]
    fn set_property_double(&mut self, prop: Property, double: f64) -> Result<()> {
        mpv_try_unknown!(prop)?;
        let mut double: libc::c_double = double;
        mpv_try!(unsafe {
            mpv_set_property(
                self.ctx,
                prop.as_ptr(),
                Format::Double.to_int(),
                ptr::from_mut(&mut double) as *mut libc::c_void,
            )
        })?;
        Ok(())
    }

    fn get_property_int(&mut self, prop: Property) -> Result<i64> {
        mpv_try_unknown!(prop)?;
        let mut int: int64_t = 0;
        mpv_try!(unsafe {
            mpv_get_property(
                self.ctx,
                prop.as_ptr(),
                Format::Int64.to_int(),
                ptr::from_mut(&mut int) as *mut libc::c_void,
            )
        })?;
        Ok(int)
    }

    #[allow(dead_code)]
    fn set_property_int(&mut self, prop: Property, int: i64) -> Result<()> {
        mpv_try_unknown!(prop)?;
        let mut int: int64_t = int;
        mpv_try!(unsafe {
            mpv_set_property(
                self.ctx,
                prop.as_ptr(),
                Format::Int64.to_int(),
                ptr::from_mut(&mut int) as *mut libc::c_void,
            )
        })?;
        Ok(())
    }

    fn observe_property(&mut self, prop: Property, format: Format) -> Result<()> {
        mpv_try_unknown!(prop)?;
        mpv_try_unknown!(format)?;
        mpv_try!(unsafe {
            mpv_observe_property(self.ctx, 0, prop.as_ptr(), format.to_int())
        })?;
        Ok(())
    }
}

impl Handle<Init> {
    pub fn enable_default_bindings(&mut self) -> Result<()> {
        self.set_property_flag(Property::InputDefaultBindings, true)?;
        self.set_property_flag(Property::InputVoKeyboard, true)?;
        Ok(())
    }
}

enum_cstr_map! {AudioDriver {
    (Pulse, c"pulse"),
}}

impl Handle<Uninit> {
    pub fn set_audio_driver(&mut self, device: AudioDriver) -> Result<()> {
        mpv_try_unknown!(device)?;
        self.set_property_string(Property::AudioDriver, device)
    }
}
