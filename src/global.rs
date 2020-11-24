pub mod classes {
    use crate::object::prelude::*;
    use crate::vm;

    macro_rules! PRELOADED_CLASSES {
        ($DO:tt) => {
            $DO!(CLASS_CLASS, "java/lang/Class", cclass);
            $DO!(OBJ_CLASS, "java/lang/Object", obj_class);
            $DO!(CHAR_CLASS, "char", char_class);
            $DO!(BYTE_CLASS, "byte", byte_class);
            $DO!(BOOLEAN_CLASS, "boolean", boolean_class);
            $DO!(INT_CLASS, "int", int_class);
            $DO!(SHORT_CLASS, "short", short_class);
            $DO!(LONG_CLASS, "long", long_class);
            $DO!(FLOAT_CLASS, "float", float_class);
            $DO!(DOUBLE_CLASS, "double", double_class);
            $DO!(VOID_CLASS, "void", void_class);
        };
    }

    macro_rules! PRELOADED_CLASSES_DEFINE {
        ($name:ident, $class:expr, $access:ident) => {
            static mut $name: ClassPtr = ClassPtr::null();
        };
    }

    macro_rules! PRELOADED_CLASSES_INIT {
        ($name:ident, $class:expr, $access:ident) => {
            $name = new_permanent_class($class);
        };
    }

    macro_rules! PRELOADED_CLASS_ACCESSOR {
        ($name:ident, $class:expr, $accessor:ident) => {
            pub fn $accessor() -> ClassPtr {
                unsafe { $name }
            }
        };
    }

    PRELOADED_CLASSES!(PRELOADED_CLASSES_DEFINE);

    PRELOADED_CLASSES!(PRELOADED_CLASS_ACCESSOR);

    pub fn init() {
        unsafe {
            PRELOADED_CLASSES!(PRELOADED_CLASSES_INIT);
            CLASS_CLASS.bootstrap(CLASS_CLASS);
        }
    }

    fn new_permanent_class(name: &str) -> ClassPtr {
        let name = vm::instance()
            .symbol_table
            .get_or_insert(String::from(name));
        let flags = ClassAccessFlags::AccPublic as u16 | ClassAccessFlags::AccFinal as u16;
        return Class::new_permanent(
            ConstantPoolPtr::null(),
            flags,
            name,
            ClassPtr::null(),
            JRefArrayPtr::null(),
            FieldArrayPtr::null(),
            MethodArrayPtr::null(),
        );
    }
}
