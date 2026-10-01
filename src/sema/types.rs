use crate::ast::Type;

pub fn is_assignable(target: &Type, source: &Type) -> bool {
    if target == source {
        return true;
    }

    match (target, source) {
        // Numeric promotions / conversions between integer types
        (Type::I32, Type::I8) | (Type::I32, Type::I16) | (Type::I32, Type::U8) | (Type::I32, Type::U16) => true,
        (Type::U32, Type::U8) | (Type::U32, Type::U16) => true,
        // Any pointer can convert to/from *u8 or void pointer (or integer 0/null in i32)
        (Type::Pointer(_), Type::Pointer(_)) => true,
        (Type::Pointer(_), Type::I32) | (Type::Pointer(_), Type::U32) => true,
        (Type::I32, Type::Pointer(_)) | (Type::U32, Type::Pointer(_)) => true,
        _ => false,
    }
}
