use crate::tunables::Tunables;
use std::sync::Arc;
#[cfg(feature = "compiler")]
use wasmer_compiler::CompilerConfig;
use wasmer_engine::Engine;

#[derive(Clone)]
pub struct Store {
    engine: Arc<dyn Engine + Send + Sync>,
}

impl Store {
    pub fn new(engine: Arc<dyn Engine + Send + Sync>) -> Store {
        Store { engine }
    }

    pub fn engine(&self) -> &Arc<dyn Engine + Send + Sync> {
        &self.engine
    }

    pub fn same(a: &Store, b: &Store) -> bool {
        Arc::ptr_eq(&a.engine, &b.engine)
    }
}

impl PartialEq for Store {
    fn eq(&self, other: &Self) -> bool {
        Store::same(self, other)
    }
}

// We only implement default if we have assigned a default compiler and engine
#[cfg(all(feature = "compiler", feature = "engine"))]
impl Default for Store {
    fn default() -> Store {
        // We store them on a function that returns to make
        // sure this function doesn't emit a compile error even if
        // more than one compiler is enabled.
        #[allow(unreachable_code)]
        fn get_config() -> Arc<dyn CompilerConfig + Send + Sync> {
            #[cfg(feature = "cranelift")]
            return Arc::new(wasmer_compiler_cranelift::CraneliftConfig::default());

            #[cfg(feature = "llvm")]
            return Arc::new(wasmer_compiler_llvm::LLVMConfig::default());

            #[cfg(feature = "singlepass")]
            return Arc::new(wasmer_compiler_singlepass::SinglepassConfig::default());
        }

        #[allow(unreachable_code)]
        fn get_engine(
            config: Arc<dyn CompilerConfig + Send + Sync>,
        ) -> Arc<dyn Engine + Send + Sync> {
            let tunables = Tunables::for_target(config.target().triple());

            #[cfg(feature = "jit")]
            return Arc::new(wasmer_engine_jit::JITEngine::new(&*config, tunables));
        }

        let config = get_config();
        let engine = get_engine(config);
        Store::new(engine)
    }
}

pub trait StoreObject {
    fn comes_from_same_store(&self, store: &Store) -> bool;
}
