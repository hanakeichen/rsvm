pub use super::array::{
    JBooleanArrayPtr, JByteArray, JByteArrayPtr, JCharArrayPtr, JDoubleArrayPtr, JFloatArrayPtr,
    JIntArrayPtr, JLongArrayPtr, JRefArray, JRefArrayPtr, JShortArrayPtr,
};
pub use super::class::{
    Class, ClassAccessFlags, ClassPtr, ConstantPoolPtr, FieldArrayPtr, MethodArrayPtr,
};
pub use super::ptr::Ptr;
pub use super::symbol::{SymbolPtr, SymbolTable};
pub use super::Object;

pub type JChar = u16;
pub type JByte = i8;
pub type JBoolean = JByte;
pub type JShort = i16;
pub type JInt = i32;
pub type JLong = i64;

pub type JFloat = f32;
pub type JDouble = f64;

pub type ObjectPtr = Ptr<Object>;
