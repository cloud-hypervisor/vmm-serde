// Copyright (C) 2019 Alibaba Cloud. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 or BSD-3-Clause

//! Helper macros to opt-in/opt-out serde.
//!
//! A common requirement of VM snapshot, VM live upgrading and VM live migration is to serialize and
//! deserialize VMM internal objects and states. This crate provides helper macros to help serialize
//! and deserialize VMM internal objects and states.
//!
//! # Procedural Macro to Export Private Fields of Struct as Public
//! To hide crate/module implementation details from the crate/module users, it's suggested to mark
//! all internal fields as private. But when enabling the VM snapshot/live upgrading/live migration,
//! the snapshot/live upgrading/live migration subsystem often needs to access other crate/module's
//! internal state. In other words, the snapshot/live upgrading/live migration subsystem needs to
//! access struct fields normally marked as private. So the export_as_pub proc macro is introduced
//! to mark struct fields as pub when the `export_as_pub` feature is enabled. Otherwise the
//! export_as_pub proc macro translates to a noop.
//!
//! ## Example
//! Suppose we have a Struct defined as:
//! ```
//! # extern crate vmm_serde;
//!
//! #[vmm_serde::export_as_pub()]
//! pub(crate) struct VmmObject {
//!     state: u32,
//!     pub(crate) features: u64,
//! }
//! ```
//!
//! When the `export_as_pub` feature is enabled, the Struct will be translated as:
//! ```
//! #[vmm_serde::export_as_pub()]
//! pub struct VmmObject {
//!     pub state: u32,
//!     pub features: u64,
//! }
//! ```
//!
//! And when the `export_as_pub` feature is disabled, the Struct will be translated as:
//! ```
//! pub(crate) struct VmmObject {
//!     state: u32,
//!     pub(crate) features: u64,
//! }
//! ```
//!
//! Instead of exporting all fields as pub, user may specify the fields needed to be pub as:
//! ```
//! # extern crate vmm_serde;
//!
//! #[vmm_serde::export_as_pub(features)]
//! pub(crate) struct VmmObject {
//!     state: u32,
//!     pub(crate) features: u64,
//! }
//! ```
//!
//! # Control #[derive(Serialize, Deserialize)] by Feature
//! The serde_derive crate exports proc_macro_derive(Serialize, Deserialize) to support the serde
//! crate, but it does introduce heavy dependency chains. So introduce the feature `serde_derive`.
//! When the feature `serde_derive` is enabled, implementation of #[derive(Serialize, Deserialize)]
//! is reexported from the serde_derive crate. When the feature `serde_derive` is disabled, a noop
//! implementation for  #[derive(Serialize, Deserialize)] is provided.
//!
//! ## Example
//! ```
//! # extern crate vmm_serde;
//! # use vmm_serde::{Serialize, Deserialize};
//!
//! #[derive(Serialize, Deserialize)]
//! pub struct VmmObject {
//!     state: u32,
//! }
//! ```
//!
//! # Support Serialization/Deserialization for FFI Data Structures
//! Often the bindgen utiltily is used to auto-generate FFI bindings for Linux syscalls and system
//! libraries. And data structures with flexible array member are often used for input and/or output
//! parameters. So three derive macros, SerializeFfi, DeserializeFfi and DeserializeFfiFam, are
//! introduced to serialize data structures generated by bindgen.
//!
//! ## Exmaple
//! - Serialize/Deserialize Fix-sized FFI Data Structure
//! ```
//! # extern crate vmm_serde;
//! # use vmm_serde::*;
//!
//! #[repr(C)]
//! #[derive(Debug, Default, Copy, Clone, PartialEq, SerializeFfi, DeserializeFfi)]
//! pub struct kvm_memory_alias {
//!    pub slot: u32,
//!    pub flags: u32,
//!    pub guest_phys_addr: u64,
//!    pub memory_size: u64,
//!    pub target_phys_addr: u64,
//! }
//! ```
//!
//! - Serialize/Deserialize FFI Data Structure with Flexible Array Member
//! ```
//! # extern crate vmm_serde;
//! # use vmm_serde::*;
//!
//! #[repr(C)]
//! #[derive(Default, Debug, SerializeFfi, DeserializeFfi)]
//! pub struct __IncompleteArrayField<T>(::std::marker::PhantomData<T>, [T; 0]);
//! impl<T> __IncompleteArrayField<T> {
//!    #[inline]
//!    pub fn new() -> Self {
//!        __IncompleteArrayField(::std::marker::PhantomData, [])
//!    }
//! }
//!
//! #[repr(C)]
//! #[derive(Debug, Default, SerializeFfi, DeserializeFfiFam)]
//! pub struct kvm_msrs {
//!    pub nmsrs: u32,
//!    pub pad: u32,
//!    pub entries: __IncompleteArrayField<u64>,
//! }
//!
//! #[cfg(feature = "serde_derive_ffi")]
//! impl SizeofFamStruct for kvm_msrs {
//!    fn size_of(&self) -> usize {
//!         self.nmsrs as usize * std::mem::size_of::<u64>() + std::mem::size_of::<Self>()
//!    }
//! }
//! ```

#[cfg(feature = "serde_derive")]
#[doc(hidden)]
pub use serde::de::Error as VmmSerdeError;
#[cfg(feature = "serde_derive")]
#[doc(hidden)]
pub use serde::*;

#[cfg(feature = "serde_derive_ffi")]
mod ffi;
#[cfg(feature = "serde_derive_ffi")]
pub use ffi::{deserialize_ffi, deserialize_ffi_fam, serialize_ffi, ByteBuf, SizeofFamStruct};

#[doc(hidden)]
pub use vmm_serde_impl::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(dead_code)]
    fn test_export_as_pub() {
        #[allow(unused_imports)]
        #[export_as_pub()]
        use std::result::Result;

        #[export_as_pub()]
        pub(super) struct VmmObject1 {
            state: u32,
            pub(crate) features: u64,
        }

        #[export_as_pub(state)]
        pub struct VmmObject2 {
            state: u32,
            pub(crate) features: u64,
        }

        #[export_as_pub(features)]
        struct VmmObject3 {
            state: u32,
            pub(crate) features: u64,
        }

        #[export_as_pub(state, features)]
        struct VmmObject4 {
            state: u32,
            pub(crate) features: u64,
        }
    }

    #[test]
    #[allow(dead_code)]
    fn test_derive() {
        #[derive(Serialize, Deserialize)]
        pub(super) struct VmmObject5 {
            state: u32,
        }

        #[derive(Serialize)]
        pub(super) struct VmmObject6 {
            state: u32,
        }

        #[derive(Deserialize)]
        pub(super) struct VmmObject7 {
            state: u32,
        }
    }
}
