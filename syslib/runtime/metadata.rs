/*
 * Nuva OS - SystemLibrary - Runtime
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

//! Type Metadata System

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Type Flags
pub const TYPE_FLAG_CLASS: u32 = 1 << 0;
pub const TYPE_FLAG_STRUCT: u32 = 1 << 1;
pub const TYPE_FLAG_ENUM: u32 = 1 << 2;
pub const TYPE_FLAG_PROTOCOL: u32 = 1 << 3;
pub const TYPE_FLAG_GENERIC: u32 = 1 << 4;
pub const TYPE_FLAG_FINAL: u32 = 1 << 5;
pub const TYPE_FLAG_ABSTRACT: u32 = 1 << 6;

/// Type Metadata
#[derive(Debug)]
#[repr(C)]
pub struct TypeMetadata {
    /// Type name
    pub name: [u8; 64],
    pub name_len: u8,

    /// Type flags
    pub flags: AtomicU32,

    /// Type size
    pub size: u32,

    /// Alignment requirement
    pub alignment: u32,

    /// Parent type
    pub super_type: AtomicU64,

    /// Protocol witness table
    pub protocol_witnesses: AtomicU64,

    /// Virtual function table
    pub vtable: AtomicU64,

    /// Property info
    pub properties: AtomicU64,
    pub num_properties: AtomicU32,

    /// Method info
    pub methods: AtomicU64,
    pub num_methods: AtomicU32,

    /// Type ID
    pub type_id: AtomicU64,
}

impl Clone for TypeMetadata {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            name_len: self.name_len.clone(),
            flags: AtomicU32::new(self.flags.load(core::sync::atomic::Ordering::Relaxed)),
            size: self.size.clone(),
            alignment: self.alignment.clone(),
            super_type: AtomicU64::new(self.super_type.load(core::sync::atomic::Ordering::Relaxed)),
            protocol_witnesses: AtomicU64::new(self.protocol_witnesses.load(core::sync::atomic::Ordering::Relaxed)),
            vtable: AtomicU64::new(self.vtable.load(core::sync::atomic::Ordering::Relaxed)),
            properties: AtomicU64::new(self.properties.load(core::sync::atomic::Ordering::Relaxed)),
            num_properties: AtomicU32::new(self.num_properties.load(core::sync::atomic::Ordering::Relaxed)),
            methods: AtomicU64::new(self.methods.load(core::sync::atomic::Ordering::Relaxed)),
            num_methods: AtomicU32::new(self.num_methods.load(core::sync::atomic::Ordering::Relaxed)),
            type_id: AtomicU64::new(self.type_id.load(core::sync::atomic::Ordering::Relaxed)),
        }
    }
}

impl TypeMetadata {
    pub fn new(name: &[u8], size: u32, alignment: u32) -> Self {
        let mut name_buf = [0u8; 64];
        let len = name.len().min(63);
        name_buf[..len].copy_from_slice(&name[..len]);

        Self {
            name: name_buf,
            name_len: len as u8,
            flags: AtomicU32::new(0),
            size,
            alignment,
            super_type: AtomicU64::new(0),
            protocol_witnesses: AtomicU64::new(0),
            vtable: AtomicU64::new(0),
            properties: AtomicU64::new(0),
            num_properties: AtomicU32::new(0),
            methods: AtomicU64::new(0),
            num_methods: AtomicU32::new(0),
            type_id: AtomicU64::new(0),
        }
    }

    pub fn is_class(&self) -> bool {
        self.flags.load(Ordering::Relaxed) & TYPE_FLAG_CLASS != 0
    }

    pub fn is_struct(&self) -> bool {
        self.flags.load(Ordering::Relaxed) & TYPE_FLAG_STRUCT != 0
    }

    pub fn is_enum(&self) -> bool {
        self.flags.load(Ordering::Relaxed) & TYPE_FLAG_ENUM != 0
    }

    pub fn is_protocol(&self) -> bool {
        self.flags.load(Ordering::Relaxed) & TYPE_FLAG_PROTOCOL != 0
    }

    pub fn set_flag(&self, flag: u32) {
        self.flags.fetch_or(flag, Ordering::Release);
    }

    pub fn name(&self) -> &[u8] {
        &self.name[..self.name_len as usize]
    }
}

/// Property Metadata
#[derive(Debug, Clone)]
#[repr(C)]
pub struct PropertyMetadata {
    /// Property name
    pub name: [u8; 64],
    pub name_len: u8,

    /// Property type
    pub type_metadata: u64,

    /// Offset
    pub offset: u32,

    /// Flags
    pub flags: u32,
}

/// Property Flags
pub const PROPERTY_FLAG_MUTABLE: u32 = 1 << 0;
pub const PROPERTY_FLAG_COMPUTED: u32 = 1 << 1;
pub const PROPERTY_FLAG_LAZY: u32 = 1 << 2;

/// Method Metadata
#[derive(Debug, Clone)]
#[repr(C)]
pub struct MethodMetadata {
    /// Method name
    pub name: [u8; 64],
    pub name_len: u8,

    /// Method pointer
    pub function_ptr: u64,

    /// Parameter types
    pub param_types: [u64; 8],
    pub num_params: u8,

    /// Return type
    pub return_type: u64,

    /// Flags
    pub flags: u32,
}

/// Method Flags
pub const METHOD_FLAG_STATIC: u32 = 1 << 0;
pub const METHOD_FLAG_VIRTUAL: u32 = 1 << 1;
pub const METHOD_FLAG_ABSTRACT: u32 = 1 << 2;
pub const METHOD_FLAG_MUTATING: u32 = 1 << 3;
pub const METHOD_FLAG_ASYNC: u32 = 1 << 4;

/// Type Registry
pub struct TypeRegistry {
    types: [Option<TypeMetadata>; 1024],
    num_types: AtomicU32,
    next_type_id: AtomicU64,
}

impl TypeRegistry {
    pub const fn new() -> Self {
        Self {
            types: [const { None }; 1024],
            num_types: AtomicU32::new(0),
            next_type_id: AtomicU64::new(1),
        }
    }

    pub fn init(&mut self) {
        // Register built-in types
        self.register_builtin_types();
    }

    /// Register a type
    pub fn register(&mut self, metadata: TypeMetadata) -> u64 {
        let type_id = self.next_type_id.fetch_add(1, Ordering::Relaxed);
        metadata.type_id.store(type_id, Ordering::Relaxed);

        let idx = self.num_types.load(Ordering::Relaxed) as usize;
        if idx < 1024 {
            self.types[idx] = Some(metadata);
            self.num_types.fetch_add(1, Ordering::Relaxed);
            return type_id;
        }
        0
    }

    /// Find a type by name
    pub fn lookup(&self, name: &[u8]) -> Option<&TypeMetadata> {
        for i in 0..self.num_types.load(Ordering::Relaxed) as usize {
            if let Some(ref metadata) = self.types[i] {
                if metadata.name() == name {
                    return Some(metadata);
                }
            }
        }
        None
    }

    /// Find a type by ID
    pub fn lookup_by_id(&self, id: u64) -> Option<&TypeMetadata> {
        for i in 0..self.num_types.load(Ordering::Relaxed) as usize {
            if let Some(ref metadata) = self.types[i] {
                if metadata.type_id.load(Ordering::Relaxed) == id {
                    return Some(metadata);
                }
            }
        }
        None
    }

    fn register_builtin_types(&mut self) {
        let builtins = [
            ("Int", 8, 8),
            ("Float", 8, 8),
            ("Bool", 1, 1),
            ("String", 16, 8),
            ("Void", 0, 1),
        ];

        for (name, size, align) in builtins {
            let metadata = TypeMetadata::new(name.as_bytes(), size, align);
            self.register(metadata);
        }
    }
}

/// Global Type Registry
pub static TYPE_REGISTRY: TypeRegistry = TypeRegistry::new();
