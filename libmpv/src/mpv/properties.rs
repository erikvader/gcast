use super::{
    commands::Cmd,
    data::{AllocMpvData, Format, FromMpvData, Node, ToMpvData},
    macros::{enum_cstr_map, mpv_try, mpv_try_unknown},
    Handle, Result, Uninit,
};
use crate::bindings::*;
use crate::see_string::SeeString;
use std::{marker::PhantomData, ptr::addr_of};

fn none<T>(_val: T) -> Option<PropertyValue> {
    None
}

macro_rules! properties {
    (@inner () -> ($(($name: ident, $type:ty, $int:expr, $cstr:expr, $bool:expr, $double:expr, $node:expr))*)) => {
        // TODO: if this enum is only used in `wait_event`, then maybe consider only
        // adding variants to this enum if an observe function is generated. Figure out
        // how to do that.
        #[derive(Debug, Clone)]
        pub enum PropertyValue {
            $($name($type)),*
        }

        impl Property {
            pub(crate) fn value_i64(&self, value: i64) -> Option<PropertyValue> {
                match self {
                    $(Property::$name => $int(value)),*,
                    _ => None,
                }
            }

            pub(crate) fn value_string(&self, value: *const libc::c_char) -> Option<PropertyValue> {
                match self {
                    $(Property::$name => $cstr(value)),*,
                    _ => None,
                }
            }

            pub(crate) fn value_flag(&self, value: bool) -> Option<PropertyValue> {
                match self {
                    $(Property::$name => $bool(value)),*,
                    _ => None,
                }
            }

            pub(crate) fn value_double(&self, value: f64) -> Option<PropertyValue> {
                match self {
                    $(Property::$name => $double(value)),*,
                    _ => None,
                }
            }

            pub(crate) fn value_node(&self, value: Node) -> Option<PropertyValue> {
                match self {
                    $(Property::$name => $node(value)),*,
                    _ => None,
                }
            }
        }
    };
    (@inner ((Flag,
              $prop:ident,
              $(Get $getter:ident $(,)?)?
              $(Set $setter:ident $(,)?)?
              $(Obs $obs:ident $(,)?)?
              $(Cyc $cyc:ident $(,)?)?
    ) $($rest:tt)*) -> ($($arms:tt)*)) => {
        impl<T: super::private::InitState> Handle<T> {
            $(
                pub fn $getter(&mut self) -> Get<'_, bool> {
                    self.get_property_flag(Property::$prop)
                }
            )?
            $(
                pub fn $setter(&mut self, value: bool) -> Set<'_, bool> {
                    self.set_property_flag(Property::$prop, value)
                }
            )?
            $(
                pub fn $obs(&mut self) -> Result<()> {
                    self.observe_property(Property::$prop, Format::Flag)
                }
            )?
            $(
                pub fn $cyc(&mut self) -> Cmd<'_, 'static> {
                    self.cycle(Property::$prop)
                }
            )?
        }
        properties!{@inner ($($rest)*) -> ($($arms)* ($prop, bool, none, none, |b| Some(PropertyValue::$prop(b)), none, none))}
    };
    (@inner ((Int64,
              $prop:ident,
              $(Get $getter:ident $(,)?)?
              $(Set $setter:ident $(,)?)?
              $(Obs $obs:ident $(,)?)?
              $(Add $add:ident $(,)?)?
    ) $($rest:tt)*) -> ($($arms:tt)*)) => {
        impl<T: super::private::InitState> Handle<T> {
            $(
                pub fn $getter(&mut self) -> Get<'_, i64> {
                    self.get_property_int(Property::$prop)
                }
            )?
            $(
                pub fn $setter(&mut self, value: i64) -> Set<'_, i64> {
                    self.set_property_int(Property::$prop, value)
                }
            )?
            $(
                pub fn $obs(&mut self) -> Result<()> {
                    self.observe_property(Property::$prop, Format::Int64)
                }
            )?
            $(
                pub fn $add(&mut self, val: i64) -> Cmd<'_, 'static> {
                    self.add_int(Property::$prop, val)
                }
            )?
        }
        properties!{@inner ($($rest)*) -> ($($arms)* ($prop, i64, |i| Some(PropertyValue::$prop(i)), none, none, none, none))}
    };
    (@inner ((Double,
              $prop:ident,
              $(Get $getter:ident $(,)?)?
              $(Set $setter:ident $(,)?)?
              $(Obs $obs:ident $(,)?)?
              $(Add $add:ident $(,)?)?
    ) $($rest:tt)*) -> ($($arms:tt)*)) => {
        impl<T: super::private::InitState> Handle<T> {
            $(
                pub fn $getter(&mut self) -> Get<'_, f64> {
                    self.get_property_double(Property::$prop)
                }
            )?
            $(
                pub fn $setter(&mut self, value: f64) -> Set<'_, f64> {
                    self.set_property_double(Property::$prop, value)
                }
            )?
            $(
                pub fn $obs(&mut self) -> Result<()> {
                    self.observe_property(Property::$prop, Format::Double)
                }
            )?
            $(
                pub fn $add(&mut self, val: f64) -> Cmd<'_, 'static> {
                    self.add_double(Property::$prop, val)
                }
            )?
        }
        properties!{@inner ($($rest)*) -> ($($arms)* ($prop, f64, none, none, none, |d| Some(PropertyValue::$prop(d)), none))}
    };
    (@inner ((String,
              $prop:ident,
              $(Get $getter:ident $(,)?)?
              $(Set $setter:ident $(,)?)?
              $(Obs $obs:ident $(,)?)?
    ) $($rest:tt)*) -> ($($arms:tt)*)) => {
        impl<T: super::private::InitState> Handle<T> {
            $(
                pub fn $getter(&mut self) -> Get<'_, String> {
                    self.get_property_string(Property::$prop)
                }
            )?
            $(
                pub fn $setter<'a>(&mut self, value: impl Into<SeeString<'a>>) -> Set<'_, SeeString<'a>> {
                    self.set_property_string(Property::$prop, value)
                }
            )?
            $(
                pub fn $obs(&mut self) -> Result<()> {
                    self.observe_property(Property::$prop, Format::String)
                }
            )?
        }
        properties!{@inner ($($rest)*) -> ($($arms)* ($prop, String, none, |s| Some(PropertyValue::$prop(String::from_mpv_data(&s))), none, none, none))}
    };
    (@inner ((EnumCstr $enum:ident,
              $prop:ident,
              $(Get $getter:ident $(,)?)?
              $(Set $setter:ident $(,)?)?
              $(Obs $obs:ident $(,)?)?
    ) $($rest:tt)*) -> ($($arms:tt)*)) => {
        impl FromMpvData for $enum {
            type Input = *const libc::c_char;

            fn from_mpv_data(mpv: &Self::Input) -> Self {
                Self::from_ptr(*mpv)
            }
        }

        impl ToMpvData for $enum {
            type Output = *const libc::c_char;

            fn to_mpv_data(&self) -> Self::Output {
                self.as_ptr()
            }
        }

        impl<T: super::private::InitState> Handle<T> {
            $(
                pub fn $getter(&mut self) -> Get<'_, $enum> {
                    Get{
                        ctx: self.ctx,
                        _phant: PhantomData,
                        _phant2: PhantomData,
                        format: Format::String,
                        property: Property::$prop,
                    }
                }
            )?
            $(
                pub fn $setter(&mut self, value: $enum) -> Set<'_, $enum> {
                    Set {
                        data: value,
                        ctx: self.ctx,
                        _phant: PhantomData,
                        format: Format::String,
                        property: Property::$prop,
                    }
                }
            )?
            $(
                pub fn $obs(&mut self) -> Result<()> {
                    self.observe_property(Property::$prop, Format::String)
                }
            )?
        }
        properties!{@inner ($($rest)*) -> ($($arms)* ($prop, $enum, none, |s| Some(PropertyValue::$prop($enum::from_ptr(s))), none, none, none))}
    };
    (@inner ((Node,
              $prop:ident,
              $(Get $getter:ident $(,)?)?
              $(Obs $obs:ident $(,)?)?
    ) $($rest:tt)*) -> ($($arms:tt)*)) => {
        impl<T: super::private::InitState> Handle<T> {
            $(
                pub fn $getter(&mut self) -> Get<'_, Node> {
                    self.get_property_node(Property::$prop)
                }
            )?
            $(
                pub fn $obs(&mut self) -> Result<()> {
                    self.observe_property(Property::$prop, Format::Node)
                }
            )?
        }
        properties!{@inner ($($rest)*) -> ($($arms)* ($prop, Node, none, none, none, none, |n| Some(PropertyValue::$prop(n))))}
    };
    ($($rest:tt),* $(,)?) => {
        properties!{@inner ($($rest)*) -> ()}
    };
}

properties! {
    (Flag, Pause, Get is_paused, Set set_paused, Obs observe_paused, Cyc toggle_pause),
    (String, MpvVersion, Get version),
    (String, MediaTitle, Get media_title, Obs observe_media_title),
    (Double, PlaybackTime, Get playback_time, Obs observe_playback_time),
    (Double, Duration, Get duration, Obs observe_duration),
    (Double, Volume, Get volume, Set set_volume, Obs observe_volume, Add add_volume),
    (Int64, Chapters, Get chapters, Obs observe_chapters),
    (Int64, Chapter, Get chapter, Obs observe_chapter, Add add_chapter),
    (Node, TrackList, Get track_list, Obs observe_track_list),
    (String, YtdlFormat, Set set_ytdl_format),
    (Flag, Fullscreen, Set set_fullscreen),
    (Flag, Mute, Get is_muted, Obs observe_muted, Cyc toggle_mute),
    (Double, SubDelay, Add add_sub_delay),
    (Double, SubScale, Add add_sub_scale),
    (Double, SubPos, Add add_sub_pos),
    (Int64, SubId, Set set_sub),
    (Int64, AudioId, Set set_audio),
    (EnumCstr Idle, Idle, Get get_idle, Set set_idle),
}

enum_cstr_map! {pub Property {
    (MpvVersion, c"mpv-version"),
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
    (YtdlFormat, c"ytdl-format"),
    (Fullscreen, c"fullscreen"),
    (Mute, c"mute"),
    (SubDelay, c"sub-delay"),
    (SubScale, c"sub-scale"),
    (SubPos, c"sub-pos"),
    (SubId, c"sid"),
    (AudioId, c"aid"),
    (Config, c"config"),
    (ConfigDir, c"config-dir"),
    (Idle, c"idle"),
}}

enum_cstr_map! {pub Idle {
    (No, c"no"),
    (Yes, c"yes"),
    (Once, c"once"),
}}

impl<T: super::private::InitState> Handle<T> {
    pub fn enable_default_bindings(&mut self) -> Result<()> {
        self.set_property_flag(Property::InputDefaultBindings, true)
            .synch()?;
        self.set_property_flag(Property::InputVoKeyboard, true)
            .synch()?;
        Ok(())
    }
}

impl Handle<Uninit> {
    pub fn read_config_file(&mut self) -> Result<()> {
        self.set_property_flag(Property::Config, true).synch()
    }

    pub fn set_config_dir<'a>(&mut self, path: impl Into<SeeString<'a>>) -> Result<()> {
        // NOTE: hopefully mpv won't mangle the string if it is a file path
        self.set_property_string(Property::ConfigDir, path).synch()
    }
}

#[must_use = "must call it for something to happen"]
#[allow(private_bounds)]
pub struct Set<'handle, T>
where
    T: ToMpvData,
{
    data: T,
    ctx: *mut mpv_handle,
    _phant: PhantomData<&'handle mut mpv_handle>,
    format: Format,
    property: Property,
}

#[allow(private_bounds)]
impl<'handle, T> Set<'handle, T>
where
    T: ToMpvData,
{
    pub fn synch(self) -> Result<()> {
        mpv_try_unknown!(self.format)?;
        mpv_try_unknown!(&self.property)?;

        let data = self.data.to_mpv_data();
        mpv_try! {unsafe{mpv_set_property(
            self.ctx,
            self.property.as_ptr(),
            self.format.to_int(),
            addr_of!(data).cast_mut().cast(),
        )}}?;

        Ok(())
    }

    pub fn asynch(self, userdata: u64) -> Result<()> {
        mpv_try_unknown!(self.format)?;
        mpv_try_unknown!(&self.property)?;

        let data = self.data.to_mpv_data();
        mpv_try! {unsafe{mpv_set_property_async(
            self.ctx,
            userdata,
            self.property.as_ptr(),
            self.format.to_int(),
            addr_of!(data).cast_mut().cast(),
        )}}?;

        Ok(())
    }
}

#[must_use = "must call it for something to happen"]
#[allow(private_bounds)]
pub struct Get<'handle, T>
where
    T: FromMpvData,
    T::Input: AllocMpvData,
{
    ctx: *mut mpv_handle,
    _phant: PhantomData<&'handle mut mpv_handle>,
    _phant2: PhantomData<T>,
    format: Format,
    property: Property,
}

#[allow(private_bounds)]
impl<'handle, T> Get<'handle, T>
where
    T: FromMpvData,
    T::Input: AllocMpvData,
{
    pub fn synch(self) -> Result<T> {
        mpv_try_unknown!(self.format)?;
        mpv_try_unknown!(&self.property)?;

        let data = T::Input::empty();
        mpv_try! {unsafe{mpv_get_property(
            self.ctx,
            self.property.as_ptr(),
            self.format.to_int(),
            addr_of!(data).cast_mut().cast(),
        )}}?;

        let retval = T::from_mpv_data(&data);
        data.free();

        Ok(retval)
    }

    pub fn asynch(self, userdata: u64) -> Result<()> {
        mpv_try_unknown!(self.format)?;
        mpv_try_unknown!(&self.property)?;

        mpv_try! {unsafe{mpv_get_property_async(
            self.ctx,
            userdata,
            self.property.as_ptr(),
            self.format.to_int(),
        )}}?;

        Ok(())
    }
}

impl<T: super::private::HandleState> Handle<T> {
    fn set_property_string<'a>(
        &mut self,
        prop: Property,
        value: impl Into<SeeString<'a>>,
    ) -> Set<'_, SeeString<'a>> {
        Set {
            data: value.into(),
            ctx: self.ctx,
            _phant: PhantomData,
            format: Format::String,
            property: prop,
        }
    }

    fn get_property_string(&mut self, prop: Property) -> Get<'_, String> {
        Get {
            ctx: self.ctx,
            _phant: PhantomData,
            _phant2: PhantomData,
            format: Format::String,
            property: prop,
        }
    }

    fn get_property_node(&mut self, prop: Property) -> Get<'_, Node> {
        Get {
            ctx: self.ctx,
            _phant: PhantomData,
            _phant2: PhantomData,
            format: Format::Node,
            property: prop,
        }
    }

    fn get_property_flag(&mut self, prop: Property) -> Get<'_, bool> {
        Get {
            ctx: self.ctx,
            _phant: PhantomData,
            _phant2: PhantomData,
            format: Format::Flag,
            property: prop,
        }
    }

    fn set_property_flag(&mut self, prop: Property, flag: bool) -> Set<'_, bool> {
        Set {
            data: flag,
            ctx: self.ctx,
            _phant: PhantomData,
            format: Format::Flag,
            property: prop,
        }
    }

    fn get_property_double(&mut self, prop: Property) -> Get<'_, f64> {
        Get {
            ctx: self.ctx,
            _phant: PhantomData,
            _phant2: PhantomData,
            format: Format::Double,
            property: prop,
        }
    }

    fn set_property_double(&mut self, prop: Property, double: f64) -> Set<'_, f64> {
        Set {
            data: double,
            ctx: self.ctx,
            _phant: PhantomData,
            format: Format::Double,
            property: prop,
        }
    }

    fn get_property_int(&mut self, prop: Property) -> Get<'_, i64> {
        Get {
            ctx: self.ctx,
            _phant: PhantomData,
            _phant2: PhantomData,
            format: Format::Int64,
            property: prop,
        }
    }

    fn set_property_int(&mut self, prop: Property, int: i64) -> Set<'_, i64> {
        Set {
            data: int,
            ctx: self.ctx,
            _phant: PhantomData,
            format: Format::Int64,
            property: prop,
        }
    }

    fn observe_property(&mut self, prop: Property, format: Format) -> Result<()> {
        mpv_try_unknown!(&prop)?;
        mpv_try_unknown!(format)?;
        // TODO: use the userdata
        mpv_try!(unsafe {
            mpv_observe_property(self.ctx, 0, prop.as_ptr(), format.to_int())
        })?;
        Ok(())
    }
}
