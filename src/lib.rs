#![warn(missing_docs)]
//! Wrapper around [`wasmtime-wasi`](https://docs.rs/wasmtime-wasi) that automatically detects the WASI version
//! used by the module.
//!
//! # Example
//! 
//! ```rust
//! # use auto_wasi::*;
//! # use wasmtime::*;
//! # use wasmtime_wasi::*;
//! # fn test() -> anyhow::Result<()> {
//! let wat = r#"
//! (module
//!     (type $empty (func))
//!     ;; In the real world this would be an actual wasi import,
//!     ;; but this crate only checks the module name.
//!     (import "wasi_snapshot_preview1" "" (func (type $empty)))
//!  )
//! "#;
//! let store = Store::default();
//! let ctx = WasiCtx::new(std::env::args())?;
//!
//! let wasm = wat::parse_str(wat)?;
//! let wasi = AutoWasi::detect(&store, ctx, wasm)?;
//! # Ok(()) }
//! ```
use anyhow::Result;
use wasi_common::WasiCtx;
use wasmparser::{Parser, Payload};
use wasmtime::{Func, Linker, Store};

/// An instantiated instance of the wasi exports.
///
/// This represents a wasi module which can be used to instantiate other wasm modules.
/// This structure exports all that various fields of the wasi instance as fields which can be used to implement your own instantiation logic, if necessary.
/// Additionally [`AutoWasi::get_export`](crate::AutoWasi::get_export) can be used to do name-based resolution.
pub enum AutoWasi {
    /// WASI imports for the old `wasi_unstable` import module.
    Snapshot0(wasmtime_wasi::old::snapshot_0::Wasi),
    /// WASI imports for the current `wasi_snapshot_preview1` import module.
    Snapshot1(wasmtime_wasi::Wasi),
}

impl AutoWasi {
    /// Creates a new [`AutoWasi`](crate::AutoWasi) that allows for linking from the detected
    /// wasi version.
    pub fn detect<T: AsRef<[u8]>>(store: &Store, ctx: WasiCtx, binary: T) -> Result<Self> {
        let version = WasiVersion::detect(binary)?;
        Ok(Self::new(store, ctx, version))
    }

    /// Creates a new [`AutoWasi`](crate::AutoWasi) that allows for linking from the provided
    /// [`WasiVersion`](crate::WasiVersion).
    pub fn new(store: &Store, ctx: WasiCtx, version: WasiVersion) -> Self {
        match version {
            WasiVersion::Snapshot0 => {
                let wasi = wasmtime_wasi::old::snapshot_0::Wasi::new(&store, ctx);
                Self::Snapshot0(wasi)
            }
            WasiVersion::Snapshot1 => {
                let wasi = wasmtime_wasi::Wasi::new(&store, ctx);
                Self::Snapshot1(wasi)
            }
        }
    }

    /// Looks up a field called name in this structure, returning it if found.
    /// This is often useful when instantiating a wasmtime instance where name resolution often happens with strings.
    pub fn get_export(&self, name: &str) -> Option<&Func> {
        match self {
            Self::Snapshot0(wasi) => wasi.get_export(name),
            Self::Snapshot1(wasi) => wasi.get_export(name),
        }
    }

    /// Adds all instance items to the specified Linker.
    pub fn add_to_linker(&self, linker: &mut Linker) -> Result<()> {
        match self {
            Self::Snapshot0(wasi) => wasi.add_to_linker(linker),
            Self::Snapshot1(wasi) => wasi.add_to_linker(linker),
        }
    }
}

/// The version of WASI that a binary relies on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WasiVersion {
    /// Called `wasi_unstable` in binaries.
    Snapshot0,
    /// Called `wasi_snapshot_preview1` in binaries.
    Snapshot1,
}

impl WasiVersion {
    /// Detects the WASI version used by the binary, defaults to the latest.
    pub fn detect<T: AsRef<[u8]>>(binary: T) -> Result<Self> {
        for payload in Parser::new(0).parse_all(binary.as_ref()) {
            match payload? {
                Payload::ImportSection(reader) => {
                    for import in reader {
                        if import?.module == "wasi_unstable" {
                            return Ok(Self::Snapshot0);
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(Self::default())
    }
}

impl Default for WasiVersion {
    fn default() -> Self {
        Self::Snapshot1
    }
}
