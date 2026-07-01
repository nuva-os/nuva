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

use alloc::vec::Vec;
use crate::nuva_lang::parser::ast::{EnumDef, StructDef, Variant, Field};
use crate::nuva_lang::semantic::types::Type;

/// ADT Runtime Representation
/// Algebraic data types are represented as tagged unions at runtime.
/// This module handles memory layout optimization and type introspection.

/// ADT Value
/// Runtime representation of an algebraic data type value.
/// Uses tagged union representation for sum types.
#[repr(C)]
pub struct AdtValue {
    /// Tag to identify the active variant
    tag: u32,
    /// Data payload (inline or heap-allocated)
    data: AdtData,
}

/// ADT Data
/// Union of possible data representations.
/// Small values are stored inline, large values are heap-allocated.
#[repr(C)]
pub union AdtData {
    /// Inline storage for small values (up to 16 bytes)
    inline: [u8; 16],
    /// Heap pointer for large values
    heap: *mut u8,
    /// Unit value for tag-only variants
    unit: (),
}

/// ADT Layout
/// Describes the memory layout of an algebraic data type.
#[derive(Debug, Clone)]
pub struct AdtLayout {
    /// Name of the ADT
    pub name: &'static str,
    /// Size in bytes
    pub size: usize,
    /// Alignment in bytes
    pub align: usize,
    /// Variant layouts
    pub variants: Vec<VariantLayout>,
    /// Is this a sum type (enum) or product type (struct)
    pub is_sum_type: bool,
}

/// Variant Layout
/// Describes the memory layout of a single variant.
#[derive(Debug, Clone)]
pub struct VariantLayout {
    /// Variant name
    pub name: &'static str,
    /// Tag value (for sum types)
    pub tag: u32,
    /// Field layouts
    pub fields: Vec<FieldLayout>,
    /// Size in bytes
    pub size: usize,
    /// Alignment in bytes
    pub align: usize,
}

/// Field Layout
/// Describes the memory layout of a single field.
#[derive(Debug, Clone)]
pub struct FieldLayout {
    /// Field name (if named)
    pub name: Option<&'static str>,
    /// Field type
    pub field_type: Type,
    /// Offset in bytes from start of variant
    pub offset: usize,
    /// Size in bytes
    pub size: usize,
    /// Alignment in bytes
    pub align: usize,
}

/// Layout Calculator
/// Calculates optimal memory layouts for algebraic data types.
pub struct LayoutCalculator {
    /// Pointer size in bytes
    pointer_size: usize,
    /// Maximum inline size (values larger than this are heap-allocated)
    max_inline_size: usize,
}

impl LayoutCalculator {
    /// Create new layout calculator
    pub const fn new() -> Self {
        LayoutCalculator {
            pointer_size: 8, // 64-bit pointers
            max_inline_size: 16,
        }
    }

    /// Calculate layout for struct (product type)
    pub fn calculate_struct_layout(&self, struct_def: &StructDef) -> AdtLayout {
        let mut fields = Vec::new();
        let mut current_offset = 0;
        let mut max_align = 1;

        for field in &struct_def.fields {
            let field_layout = self.calculate_field_layout(field, current_offset);
            current_offset = self.align_up(current_offset + field_layout.size, field_layout.align);
            max_align = max_align.max(field_layout.align);
            fields.push(field_layout);
        }

        let size = self.align_up(current_offset, max_align);

        AdtLayout {
            name: struct_def.name,
            size,
            align: max_align,
            variants: vec![VariantLayout {
                name: struct_def.name,
                tag: 0,
                fields,
                size,
                align: max_align,
            }],
            is_sum_type: false,
        }
    }

    /// Calculate layout for enum (sum type)
    pub fn calculate_enum_layout(&self, enum_def: &EnumDef) -> AdtLayout {
        let mut variants = Vec::new();
        let mut max_variant_size = 0;
        let mut max_align = 4; // Minimum alignment for tag

        for (tag, variant) in enum_def.variants.iter().enumerate() {
            let variant_layout = self.calculate_variant_layout(variant, tag as u32);
            max_variant_size = max_variant_size.max(variant_layout.size);
            max_align = max_align.max(variant_layout.align);
            variants.push(variant_layout);
        }

        // Total size: tag (4 bytes) + aligned data
        let data_size = self.align_up(max_variant_size, max_align);
        let total_size = 4 + data_size;

        AdtLayout {
            name: enum_def.name,
            size: total_size,
            align: max_align,
            variants,
            is_sum_type: true,
        }
    }

    /// Calculate layout for a single variant
    fn calculate_variant_layout(&self, variant: &Variant, tag: u32) -> VariantLayout {
        let mut fields = Vec::new();
        let mut current_offset = 0;
        let mut max_align = 1;

        if let Some(ref data) = variant.data {
            for (i, ty) in data.iter().enumerate() {
                let field_layout = self.calculate_type_layout(ty, i, current_offset);
                current_offset = self.align_up(current_offset + field_layout.size, field_layout.align);
                max_align = max_align.max(field_layout.align);
                fields.push(field_layout);
            }
        }

        let size = self.align_up(current_offset, max_align);

        VariantLayout {
            name: variant.name,
            tag,
            fields,
            size,
            align: max_align,
        }
    }

    /// Calculate layout for a single field
    fn calculate_field_layout(&self, field: &Field, offset: usize) -> FieldLayout {
        let (size, align) = self.get_type_size_and_align(&field.field_type);

        FieldLayout {
            name: Some(field.name),
            field_type: field.field_type.clone(),
            offset,
            size,
            align,
        }
    }

    /// Calculate layout for a type at a given offset
    fn calculate_type_layout(&self, ty: &Type, index: usize, offset: usize) -> FieldLayout {
        let (size, align) = self.get_type_size_and_align(ty);

        FieldLayout {
            name: None,
            field_type: ty.clone(),
            offset,
            size,
            align,
        }
    }

    /// Get size and alignment for a type
    fn get_type_size_and_align(&self, ty: &Type) -> (usize, usize) {
        use crate::nuva_lang::semantic::types::TypeKind;

        match ty.kind {
            TypeKind::Int => (8, 8),
            TypeKind::Uint => (8, 8),
            TypeKind::Float => (8, 8),
            TypeKind::Bool => (1, 1),
            TypeKind::Char => (4, 4),
            TypeKind::String => (self.pointer_size, self.pointer_size),
            TypeKind::Array { elem_type, size } => {
                let (elem_size, elem_align) = self.get_type_size_and_align(&elem_type);
                (elem_size * size, elem_align)
            }
            TypeKind::Slice(_) => (self.pointer_size * 2, self.pointer_size),
            TypeKind::Tuple(ref types) => {
                let mut total_size = 0;
                let mut max_align = 1;
                for ty in types {
                    let (size, align) = self.get_type_size_and_align(ty);
                    total_size = self.align_up(total_size + size, align);
                    max_align = max_align.max(align);
                }
                (total_size, max_align)
            }
            TypeKind::Reference { .. } | TypeKind::Pointer(_) => {
                (self.pointer_size, self.pointer_size)
            }
            _ => (self.pointer_size, self.pointer_size), // Default to pointer size
        }
    }

    /// Align up a size to the given alignment
    fn align_up(&self, size: usize, align: usize) -> usize {
        (size + align - 1) & !(align - 1)
    }
}

/// Constructor Generator
/// Generates constructor functions for ADT variants.
pub struct ConstructorGenerator;

impl ConstructorGenerator {
    /// Generate constructor for a struct
    /// Creates a constructor function name in the format: "<StructName>_new"
    pub fn generate_struct_constructor(struct_def: &StructDef) -> &'static str {
        // The constructor name follows the pattern: StructName_new
        // This is a placeholder that returns the struct name;
        // in a full implementation, this would generate IR for:
        //   fn StructName_new(field1: T1, field2: T2, ...) -> StructName {
        //       StructName { field1, field2, ... }
        //   }
        struct_def.name
    }

    /// Generate constructor for an enum variant
    /// Creates a constructor function name in the format: "<EnumName>_<VariantName>"
    pub fn generate_variant_constructor(enum_def: &EnumDef, variant: &Variant) -> &'static str {
        // The constructor name follows the pattern: EnumName_VariantName
        // In a full implementation, this would generate IR for:
        //   fn EnumName_VariantName(data...) -> EnumName {
        //       EnumName::VariantName(data)
        //   }
        // The variant constructor sets the tag and stores associated data
        let _ = enum_def.name;
        variant.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use alloc::vec;

    #[test]
    fn test_layout_calculator_new() {
        let calc = LayoutCalculator::new();
        assert_eq!(calc.pointer_size, 8);
        assert_eq!(calc.max_inline_size, 16);
    }

    #[test]
    fn test_align_up() {
        let calc = LayoutCalculator::new();
        assert_eq!(calc.align_up(5, 4), 8);
        assert_eq!(calc.align_up(8, 4), 8);
        assert_eq!(calc.align_up(9, 4), 12);
    }
}
