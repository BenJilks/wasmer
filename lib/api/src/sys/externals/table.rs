use crate::sys::context::{AsContextMut, AsContextRef};
use crate::sys::exports::{ExportError, Exportable};
use crate::sys::externals::Extern;
use crate::sys::RuntimeError;
use crate::sys::TableType;
use crate::{ExternRef, Function, Value};
use wasmer_vm::{ContextHandle, InternalContextHandle, TableElement, VMExtern, VMTable};

/// A WebAssembly `table` instance.
///
/// The `Table` struct is an array-like structure representing a WebAssembly Table,
/// which stores function references.
///
/// A table created by the host or in WebAssembly code will be accessible and
/// mutable from both host and WebAssembly.
///
/// Spec: <https://webassembly.github.io/spec/core/exec/runtime.html#table-instances>
#[derive(Debug, Clone)]
pub struct Table {
    handle: ContextHandle<VMTable>,
}

fn set_table_item(
    table: &mut VMTable,
    item_index: u32,
    item: TableElement,
) -> Result<(), RuntimeError> {
    table.set(item_index, item).map_err(|e| e.into())
}

fn value_to_table_element(
    ctx: impl AsContextMut,
    val: Value,
) -> Result<wasmer_vm::TableElement, RuntimeError> {
    if !val.is_from_context(ctx.as_context_ref()) {
        return Err(RuntimeError::new("cannot pass Value across contexts"));
    }
    Ok(match val {
        Value::ExternRef(extern_ref) => {
            wasmer_vm::TableElement::ExternRef(extern_ref.map(|e| e.vm_externref()))
        }
        Value::FuncRef(func_ref) => {
            wasmer_vm::TableElement::FuncRef(func_ref.map(|f| f.vm_funcref(ctx)))
        }
        _ => return Err(RuntimeError::new("val is not reference")),
    })
}

fn value_from_table_element(ctx: impl AsContextMut, item: wasmer_vm::TableElement) -> Value {
    match item {
        wasmer_vm::TableElement::FuncRef(funcref) => {
            Value::FuncRef(funcref.map(|f| unsafe { Function::from_vm_funcref(ctx, f) }))
        }
        wasmer_vm::TableElement::ExternRef(extern_ref) => {
            Value::ExternRef(extern_ref.map(|e| unsafe { ExternRef::from_vm_externref(ctx, e) }))
        }
    }
}

impl Table {
    /// Creates a new `Table` with the provided [`TableType`] definition.
    ///
    /// All the elements in the table will be set to the `init` value.
    ///
    /// This function will construct the `Table` using the store
    /// [`BaseTunables`][crate::sys::BaseTunables].
    pub fn new(
        mut ctx: impl AsContextMut,
        ty: TableType,
        init: Value,
    ) -> Result<Self, RuntimeError> {
        let mut ctx = ctx.as_context_mut();
        let item = value_to_table_element(ctx.as_context_mut(), init)?;
        let tunables = ctx.store().tunables();
        let style = tunables.table_style(&ty);
        let mut table = tunables
            .create_host_table(&ty, &style)
            .map_err(RuntimeError::new)?;

        let num_elements = table.size();
        for i in 0..num_elements {
            set_table_item(&mut table, i, item.clone())?;
        }

        Ok(Self {
            handle: ContextHandle::new(ctx.as_context_mut().objects_mut(), table),
        })
    }

    /// Returns the [`TableType`] of the `Table`.
    pub fn ty(&self, ctx: impl AsContextRef) -> TableType {
        *self.handle.get(ctx.as_context_ref().objects()).ty()
    }

    /// Retrieves an element of the table at the provided `index`.
    pub fn get(&self, ctx: impl AsContextMut, index: u32) -> Option<Value> {
        let item = self.handle.get(ctx.as_context_ref().objects()).get(index)?;
        Some(value_from_table_element(ctx, item))
    }

    /// Sets an element `val` in the Table at the provided `index`.
    pub fn set(
        &self,
        mut ctx: impl AsContextMut,
        index: u32,
        val: Value,
    ) -> Result<(), RuntimeError> {
        let item = value_to_table_element(ctx.as_context_mut(), val)?;
        set_table_item(
            self.handle.get_mut(ctx.as_context_mut().objects_mut()),
            index,
            item,
        )
    }

    /// Retrieves the size of the `Table` (in elements)
    pub fn size(&self, ctx: impl AsContextRef) -> u32 {
        self.handle.get(ctx.as_context_ref().objects()).size()
    }

    /// Grows the size of the `Table` by `delta`, initializating
    /// the elements with the provided `init` value.
    ///
    /// It returns the previous size of the `Table` in case is able
    /// to grow the Table successfully.
    ///
    /// # Errors
    ///
    /// Returns an error if the `delta` is out of bounds for the table.
    pub fn grow(
        &self,
        mut ctx: impl AsContextMut,
        delta: u32,
        init: Value,
    ) -> Result<u32, RuntimeError> {
        let item = value_to_table_element(ctx.as_context_mut(), init)?;
        self.handle
            .get_mut(ctx.as_context_mut().objects_mut())
            .grow(delta, item)
            .ok_or_else(|| RuntimeError::new(format!("failed to grow table by `{}`", delta)))
    }

    /// Copies the `len` elements of `src_table` starting at `src_index`
    /// to the destination table `dst_table` at index `dst_index`.
    ///
    /// # Errors
    ///
    /// Returns an error if the range is out of bounds of either the source or
    /// destination tables.
    pub fn copy(
        mut ctx: impl AsContextMut,
        dst_table: &Self,
        dst_index: u32,
        src_table: &Self,
        src_index: u32,
        len: u32,
    ) -> Result<(), RuntimeError> {
        if dst_table.handle.context_id() != src_table.handle.context_id() {
            return Err(RuntimeError::new(
                "cross-`Context` table copies are not supported",
            ));
        }
        let mut ctx = ctx.as_context_mut();
        if dst_table.handle.internal_handle() == src_table.handle.internal_handle() {
            let table = dst_table.handle.get_mut(ctx.objects_mut());
            table.copy_within(dst_index, src_index, len)
        } else {
            let (src_table, dst_table) = ctx.objects_mut().get_2_mut(
                src_table.handle.internal_handle(),
                dst_table.handle.internal_handle(),
            );
            VMTable::copy(dst_table, src_table, dst_index, src_index, len)
        }
        .map_err(RuntimeError::from_trap)?;
        Ok(())
    }

    pub(crate) fn from_vm_extern(
        ctx: impl AsContextMut,
        internal: InternalContextHandle<VMTable>,
    ) -> Self {
        Self {
            handle: unsafe {
                ContextHandle::from_internal(ctx.as_context_ref().objects().id(), internal)
            },
        }
    }

    /// Checks whether this `Table` can be used with the given context.
    pub fn is_from_context(&self, ctx: impl AsContextRef) -> bool {
        self.handle.context_id() == ctx.as_context_ref().objects().id()
    }

    pub(crate) fn to_vm_extern(&self) -> VMExtern {
        VMExtern::Table(self.handle.internal_handle())
    }
}

impl<'a> Exportable<'a> for Table {
    fn get_self_from_extern(_extern: &'a Extern) -> Result<&'a Self, ExportError> {
        match _extern {
            Extern::Table(table) => Ok(table),
            _ => Err(ExportError::IncompatibleType),
        }
    }
}
