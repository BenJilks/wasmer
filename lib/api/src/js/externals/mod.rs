pub(crate) mod function;
mod global;
mod memory;
mod table;

pub use self::function::{
    FromToNativeWasmType, Function, HostFunction, WasmTypeList, WithEnv, WithoutEnv,
};

pub use self::global::Global;
pub use self::memory::{Memory, MemoryError};
pub use self::table::Table;

use crate::js::context::{AsContextMut, AsContextRef};
use crate::js::export::Export;
use crate::js::exports::{ExportError, Exportable};
use crate::js::store::StoreObject;
use crate::js::ExternType;
use std::fmt;

/// An `Extern` is the runtime representation of an entity that
/// can be imported or exported.
///
/// Spec: <https://webassembly.github.io/spec/core/exec/runtime.html#external-values>
#[derive(Clone)]
pub enum Extern {
    /// A external [`Function`].
    Function(Function),
    /// A external [`Global`].
    Global(Global),
    /// A external [`Table`].
    Table(Table),
    /// A external [`Memory`].
    Memory(Memory),
}

impl Extern {
    /// Return the underlying type of the inner `Extern`.
    pub fn ty(&self, ctx: &impl AsContextRef) -> ExternType {
        match self {
            Self::Function(ft) => ExternType::Function(ft.ty(ctx).clone()),
            Self::Memory(ft) => ExternType::Memory(ft.ty(ctx)),
            Self::Table(tt) => ExternType::Table(tt.ty(ctx)),
            Self::Global(gt) => ExternType::Global(gt.ty(ctx)),
        }
    }

    /// Create an `Extern` from an `wasmer_compiler::Export`.
    pub fn from_vm_export(ctx: &mut impl AsContextMut, export: Export) -> Self {
        match export {
            Export::Function(f) => Self::Function(Function::from_vm_extern(ctx, f)),
            Export::Memory(m) => Self::Memory(Memory::from_vm_extern(ctx, m)),
            Export::Global(g) => Self::Global(Global::from_vm_extern(ctx, g)),
            Export::Table(t) => Self::Table(Table::from_vm_extern(ctx, t)),
        }
    }
}

impl<'a> Exportable<'a> for Extern {
    fn get_self_from_extern(
        _ctx: &impl AsContextRef,
        _extern: &'a Self,
    ) -> Result<&'a Self, ExportError> {
        // Since this is already an extern, we can just return it.
        Ok(_extern)
    }
}

impl StoreObject for Extern {}

impl fmt::Debug for Extern {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Function(_) => "Function(...)",
                Self::Global(_) => "Global(...)",
                Self::Memory(_) => "Memory(...)",
                Self::Table(_) => "Table(...)",
            }
        )
    }
}

impl From<Function> for Extern {
    fn from(r: Function) -> Self {
        Self::Function(r)
    }
}

impl From<Global> for Extern {
    fn from(r: Global) -> Self {
        Self::Global(r)
    }
}

impl From<Memory> for Extern {
    fn from(r: Memory) -> Self {
        Self::Memory(r)
    }
}

impl From<Table> for Extern {
    fn from(r: Table) -> Self {
        Self::Table(r)
    }
}
