use crate::{
    object::{
        array::JArrayPtr,
        prelude::{JBoolean, JByte, JChar, JDouble, JFloat, JInt, JLong, JShort},
    },
    ObjectPtr,
};
use paste::paste;

macro_rules! jval_members {
    ($(($member_name:ident, $member_type:ty)),*) => {
        #[repr(C)]
        pub union JValue {
            $(
                #[allow(unused)]
                $member_name: $member_type,
            )*
        }

        impl JValue {
            paste! {
                $(
                    #[allow(unused)]
                    #[inline(always)]
                    pub fn [<with_ $member_name>]($member_name: $member_type) -> Self {
                        Self { $member_name }
                    }

                    #[allow(unused)]
                    #[inline(always)]
                    pub fn [<set_ $member_name>](&mut self, $member_name: $member_type) {
                        self.$member_name = $member_name;
                    }
                )*
            }

            $(
                #[inline(always)]
                pub fn $member_name(&self) -> $member_type {
                    unsafe { self.$member_name }
                }
            )*

        }
    };
}

jval_members!(
    (bool_val, JBoolean),
    (byte_val, JByte),
    (char_val, JChar),
    (short_val, JShort),
    (int_val, JInt),
    (long_val, JLong),
    (float_val, JFloat),
    (double_val, JDouble),
    (obj_val, ObjectPtr),
    (arr_val, JArrayPtr),
    (ushort_val, u16)
);

impl JValue {
    #[inline(always)]
    pub fn with_obj_null() -> Self {
        return Self::with_obj_val(ObjectPtr::null());
    }
}

impl Default for JValue {
    fn default() -> Self {
        Self { int_val: 0 }
    }
}
