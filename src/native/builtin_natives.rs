use std::collections::HashMap;

use crate::memory::Address;

use super::{
    java_io_FileDescriptor, java_io_FileInputStream, java_io_FileOutputStream, java_io_FileSystem,
    java_io_UnixFileSystem, java_io_Win32FileSystem, java_io_WinNTFileSystem, java_lang_Class,
    java_lang_ClassLoader, java_lang_Double, java_lang_Float, java_lang_Object, java_lang_Runtime,
    java_lang_String, java_lang_System, java_lang_Thread, java_security_AccessController,
    java_util_concurrent_atomic_AtomicLong, sun_io_Win32ErrorMode, sun_misc_Signal,
    sun_misc_Unsafe, sun_misc_VM, sun_reflect_NativeConstructorAccessorImpl,
    sun_reflect_Reflection,
};
use paste::paste;

macro_rules! builtin_native_functions {
    ($(
        {$cls_name: ident, [$($inner_cls_name:ident)*], $native_fn_name: ident}
    ), *) => {
        impl BuiltinNativeFunctions {
            pub fn new() -> Self {
                let mut fns = HashMap::with_capacity(Self::num_of_natives());
                paste! {
                    $(
                        fns.insert(
                            concat!(
                                "Java_",
                                stringify!($cls_name),
                                $("$", stringify!($inner_cls_name),)*
                                "_", stringify!($native_fn_name)
                            ),
                            Address::new($cls_name::[<Java_  $cls_name $(_ $inner_cls_name)* _ $native_fn_name>] as *const u8),
                        );
                    )*
                }

                Self { fns }
            }

            const fn num_of_natives() -> usize {
                let mut num = 0;
                paste! {
                    $(
                        $cls_name::[<Java_  $cls_name $(_ $inner_cls_name)* _ $native_fn_name>] as *const u8;
                        num += 1;
                    )*
                }
                return num;
            }
        }
    };
}

builtin_native_functions!(
    {java_lang_Class, [], registerNatives},
    {java_lang_Class, [], forName0},
    {java_lang_Class, [], isInstance},
    {java_lang_Class, [], isAssignableFrom},
    {java_lang_Class, [], isInterface},
    {java_lang_Class, [], isArray},
    {java_lang_Class, [], isPrimitive},
    {java_lang_Class, [], getName0},
    {java_lang_Class, [], getClassLoader0},
    {java_lang_Class, [], getSuperclass},
    {java_lang_Class, [], getInterfaces},
    {java_lang_Class, [], getComponentType},
    {java_lang_Class, [], getModifiers},
    {java_lang_Class, [], getSigners},
    {java_lang_Class, [], setSigners},
    {java_lang_Class, [], getEnclosingMethods},
    {java_lang_Class, [], getDeclaringClass},
    {java_lang_Class, [], getProtectionDomain0},
    {java_lang_Class, [], setProtectionDomain0},
    {java_lang_Class, [], getPrimitiveClass},
    {java_lang_Class, [], getGenericSignature},
    {java_lang_Class, [], getRawAnnotations},
    {java_lang_Class, [], getConstantPool},
    {java_lang_Class, [], getDeclaredFields0},
    {java_lang_Class, [], getDeclaredMethods0},
    {java_lang_Class, [], getDeclaredConstructors0},
    {java_lang_Class, [], getDeclaredClasses0},
    {java_lang_Class, [], desiredAssertionStatus0},
    {java_lang_ClassLoader, [], registerNatives},
    {java_lang_ClassLoader, [NativeLibrary], load},
    {java_lang_System, [], registerNatives},
    {java_lang_System, [], setIn0},
    {java_lang_System, [], setOut0},
    {java_lang_System, [], setErr0},
    {java_lang_System, [], currentTimeMillis},
    {java_lang_System, [], nanoTime},
    {java_lang_System, [], arraycopy},
    {java_lang_System, [], identityHashCode},
    {java_lang_System, [], initProperties},
    {java_lang_System, [], mapLibraryName},
    {java_lang_Object, [], registerNatives},
    {java_lang_Object, [], getClass},
    {java_lang_Object, [], hashCode},
    {java_lang_Object, [], clone},
    {java_lang_Object, [], notify},
    {java_lang_Object, [], notifyAll},
    {java_lang_Object, [], wait},
    {java_lang_String, [], intern},
    {java_lang_Float, [], floatToRawIntBits},
    {java_lang_Float, [], intBitsToFloat},
    {java_lang_Double, [], doubleToRawLongBits},
    {java_lang_Double, [], longBitsToDouble},
    {java_lang_Thread, [], registerNatives},
    {java_lang_Thread, [], currentThread},
    {java_lang_Thread, [], setPriority0},
    {java_lang_Runtime, [], availableProcessors},
    {java_lang_Runtime, [], freeMemory},
    {java_io_FileInputStream, [], initIDs},
    {java_io_FileOutputStream, [], initIDs},
    {java_io_FileOutputStream, [], writeBytes},
    {java_io_FileDescriptor, [], initIDs},
    {java_io_FileDescriptor, [], set},
    {java_io_FileSystem, [], getFileSystem},

    {java_io_UnixFileSystem, [], initIDs},
    {java_io_UnixFileSystem, [], getBooleanAttributes0},
    {java_io_UnixFileSystem, [], canonicalize0},

    {java_io_WinNTFileSystem, [], initIDs},
    {java_io_WinNTFileSystem, [], getBooleanAttributes},
    {java_io_WinNTFileSystem, [], canonicalize0},

    {java_io_Win32FileSystem, [], initIDs},

    {sun_io_Win32ErrorMode, [], setErrorMode},

    {java_util_concurrent_atomic_AtomicLong, [], VMSupportsCS8},

    {java_security_AccessController, [], doPrivileged},
    {java_security_AccessController, [], getStackAccessControlContext},
    {sun_reflect_Reflection, [], getCallerClass},
    {sun_reflect_Reflection, [], getClassAccessFlags},
    {sun_reflect_NativeConstructorAccessorImpl, [], newInstance0},
    {sun_misc_Unsafe, [], registerNatives},
    {sun_misc_Unsafe, [], getByte},
    {sun_misc_Unsafe, [], putLong},
    {sun_misc_Unsafe, [], allocateMemory},
    {sun_misc_Unsafe, [], freeMemory},
    {sun_misc_Unsafe, [], objectFieldOffset},
    {sun_misc_Unsafe, [], arrayBaseOffset},
    {sun_misc_Unsafe, [], arrayIndexScale},
    {sun_misc_Unsafe, [], addressSize},
    {sun_misc_Unsafe, [], compareAndSwapObject},
    {sun_misc_Unsafe, [], compareAndSwapInt},
    {sun_misc_Unsafe, [], putOrderedObject},
    {sun_misc_Signal, [], findSignal},
    {sun_misc_Signal, [], handle0},
    {sun_misc_VM, [], initialize}
);

pub(crate) struct BuiltinNativeFunctions {
    fns: HashMap<&'static str, Address>,
}

impl BuiltinNativeFunctions {
    pub(crate) fn get_builtin_native_fn(&self, fn_name: &str) -> Option<Address> {
        return self.fns.get(fn_name).copied();
    }
}
