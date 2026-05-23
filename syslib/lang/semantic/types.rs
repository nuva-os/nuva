/*
 * Nuva OS - SystemLibrary - Lang
 * 
 * Copyright (C) 2026 Nuva OS Team
 * 
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.

 */


/// TypekindClass
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeKind {
 /// IntegerType
 Int(IntSize),
 /// noneSignIntegerType
 Uint(UintSize),
 /// DotType
 Float(FloatSize),
 /// booleanType
 Bool,
 /// CharacterType
 Char,
 /// StringType
 String,
 /// ArrayType
 Array { elem_type: Box<Type>, size: usize },
 /// SlicingType
 Slice(Box<Type>),
 /// GroupType
 Tuple(Vec<Type>),
 /// FunctionType
 Function { params: Vec<Type>, return_type: Box<Type> },
 /// StructType
 Struct { name: &'static str, fields: Vec<(&'static str, Type)> },
 /// EnumType
 Enum { name: &'static str, variants: Vec<(&'static str, Option<Vec<Type>>)> },
 /// referenceType
 Reference { inner: Box<Type>, is_mut: bool },
 /// pointerType
 Pointer { inner: Box<Type>, is_mut: bool },
 /// optionalType
 Optional(Box<Type>),
 /// resultType
 Result { ok_type: Box<Type>, err_type: Box<Type> },
 /// TypeParameter
 TypeParam(&'static str),
 /// Never Type
 Never,
 /// Type
 Unknown,
}

/// IntegerSize
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntSize {
 I8, I16, I32, I64, I128, Isize,
}

/// noneSignIntegerSize
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UintSize {
 U8, U16, U32, U64, U128, Usize,
}

/// DotSize
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatSize {
 F32, F64,
}

/// Type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Type {
 /// TypekindClass
 pub kind: TypeKind,
 /// TypeSize (Byte)
 pub size: usize,
 /// TypeAlignment (Byte)
 pub align: usize,
}

impl Type {
 /// CreatenewType
 pub const fn new(kind: TypeKind, size: usize, align: usize) -> Self {
 Type { kind, size, align }
 }
 
 /// CreateIntegerType
 pub fn int(size: IntSize) -> Self {
 let (s, a) = match size {
 IntSize::I8 => (1, 1),
 IntSize::I16 => (2, 2),
 IntSize::I32 => (4, 4),
 IntSize::I64 => (8, 8),
 IntSize::I128 => (16, 16),
 IntSize::Isize => (8, 8),
 };
 Type::new(TypeKind::Int(size), s, a)
 }
 
 /// CreateDotType
 pub fn float(size: FloatSize) -> Self {
 let (s, a) = match size {
 FloatSize::F32 => (4, 4),
 FloatSize::F64 => (8, 8),
 };
 Type::new(TypeKind::Float(size), s, a)
 }
 
 /// CreatebooleanType
 pub fn bool() -> Self {
 Type::new(TypeKind::Bool, 1, 1)
 }
 
 /// CreateCharacterType
 pub fn char() -> Self {
 Type::new(TypeKind::Char, 4, 4)
 }
 
 /// CreateStringType
 pub fn string() -> Self {
 Type::new(TypeKind::String, 24, 8) // pointer
 }
 
 /// CheckifasnumbervalueType
 pub fn is_numeric(&self) -> bool {
 matches!(self.kind, TypeKind::Int(_) | TypeKind::Uint(_) | TypeKind::Float(_))
 }
 
 /// CheckifasIntegerType
 pub fn is_integer(&self) -> bool {
 matches!(self.kind, TypeKind::Int(_) | TypeKind::Uint(_))
 }
 
 /// CheckifasDotType
 pub fn is_float(&self) -> bool {
 matches!(self.kind, TypeKind::Float(_))
 }
 
 /// CheckifasbooleanType
 pub fn is_bool(&self) -> bool {
 self.kind == TypeKind::Bool
 }
 
 /// CheckifasreferenceType
 pub fn is_reference(&self) -> bool {
 matches!(self.kind, TypeKind::Reference { .. })
 }
 
 /// CheckifasoptionalType
 pub fn is_optional(&self) -> bool {
 matches!(self.kind, TypeKind::Optional(_))
 }
 
 /// CheckifcanwithimplicitstyleconvertasotheraitemType
 pub fn can_implicit_cast_to(&self, other: &Type) -> bool {
 // mutualsameType
 if self == other {
 return true;
 }
 
 // numbervalueTypeconvert
 if self.is_numeric() && other.is_numeric() {
 // smallTypecanwithimplicitstyleconvertaslargeType
 return self.size <= other.size;
 }
 
 // optionalType
 if let TypeKind::Optional(inner) = &self.kind {
 return inner.as_ref() == other;
 }
 
 false
 }
 
 /// CheckifcanwithexplicitstyleconvertasotheraitemType
 pub fn can_explicit_cast_to(&self, other: &Type) -> bool {
 // mutualsameType
 if self == other {
 return true;
 }
 
 // numbervalueTypeofbetweencanwithexplicitstyleconvert
 if self.is_numeric() && other.is_numeric() {
 return true;
 }
 
 // pointerTypeofbetweencanwithexplicitstyleconvert
 if matches!(self.kind, TypeKind::Pointer { .. }) && matches!(other.kind, TypeKind::Pointer { .. }) {
 return true;
 }
 
 false
 }
}

/// TypeRingenvironment
pub struct TypeEnv {
 /// TypeVariablebind
 bindings: [Option<(&'static str, Type)>; 64],
 /// Bindcount
 num_bindings: u32,
}

impl TypeEnv {
 pub const fn new() -> Self {
 TypeEnv {
 bindings: [None; 64],
 num_bindings: 0,
 }
 }
 
 /// addbind
 pub fn add(&mut self, name: &'static str, ty: Type) {
 for slot in self.bindings.iter_mut() {
 if slot.is_none() {
 *slot = Some((name, ty));
 self.num_bindings += 1;
 return;
 }
 }
 }
 
 /// Findbind
 pub fn find(&self, name: &str) -> Option<&Type> {
 for slot in self.bindings.iter() {
 if let Some((n, ty)) = slot {
 if *n == name {
 return Some(ty);
 }
 }
 }
 None
 }
}