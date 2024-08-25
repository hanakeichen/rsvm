pub use super::array::{
    JArray, JByteArray, JByteArrayPtr, JCharArrayPtr, JDoubleArrayPtr, JFloatArrayPtr,
    JIntArrayPtr, JLongArrayPtr, JShortArrayPtr,
};
pub use super::class::{ClassAccessFlags, JClass, JClassPtr};
pub use super::ptr::Ptr;
pub use super::symbol::SymbolPtr;
pub use super::Object;

pub type JChar = i16;
pub type JByte = i8;
pub type JBoolean = JByte;
pub type JShort = i16;
pub type JInt = i32;
pub type JLong = i64;

pub type JFloat = f32;
pub type JDouble = f64;

pub type ObjectPtr = Ptr<Object>;
pub type ObjectRawPtr = *mut Object;
