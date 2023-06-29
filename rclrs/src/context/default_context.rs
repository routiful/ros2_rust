use std::sync::{Arc, Mutex};
use once_cell::sync::OnceCell;

use crate::context::Context;

/// Instances global default context
///
pub struct DefaultContext {
    /// Arc<Context>
    pub global_default_context: Arc<Mutex<Context>>,
}

impl DefaultContext {
    /// Returns global default context
    ///
    /// Any threads can call this then it returns the identical context.
    pub fn get_global_default_context(args: impl IntoIterator<Item = String>) -> &'static Mutex<DefaultContext> {
        static GLOBAL_DEFAULT_CONTEXT: OnceCell<Mutex<DefaultContext>> = OnceCell::new();
        GLOBAL_DEFAULT_CONTEXT.get_or_init(
            || Mutex::new(DefaultContext {
                global_default_context: Arc::new(Context::new(args).unwrap().into())
            })
        )
    }
}
