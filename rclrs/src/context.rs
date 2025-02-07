mod builder;
mod default_context;
use std::string::String;
use std::sync::{Arc, Mutex};

pub use self::builder::*;
pub use self::default_context::DefaultContext;
use crate::rcl_bindings::*;
use crate::error::{to_rclrs_result, RclrsError};

impl Drop for rcl_context_t {
    fn drop(&mut self) {
        unsafe {
            // The context may be invalid when rcl_init failed, e.g. because of invalid command
            // line arguments.
            // SAFETY: No preconditions for this function.
            if rcl_context_is_valid(self) {
                rcl_shutdown(self);
            }

            let ret = rcl_context_fini(self);
            if let Err(e) = to_rclrs_result(ret) {
                panic!("Failed to finalize context: {:?}", e);
            }
        }
    }
}

// SAFETY: The functions accessing this type, including drop(), shouldn't care about the thread
// they are running in. Therefore, this type can be safely sent to another thread.
unsafe impl Send for rcl_context_t {}

/// Shared state between nodes and similar entities.
///
/// It is possible, but not usually necessary, to have several contexts in an application.
///
/// Ownership of the context is shared by the `Context` itself and all nodes created from it.
///
/// # Details
/// A context stores, among other things
/// - command line arguments (used for e.g. name remapping)
/// - middleware-specific data, e.g. the domain participant in DDS
/// - the allocator used (left as the default by `rclrs`)
///
pub struct Context {
    pub(crate) rcl_context_mtx: Arc<Mutex<rcl_context_t>>,
    pub(crate) shutdown_callback: Option<Box<dyn Fn() + Send + Sync>>,
}

impl Context {
    /// See [`ContextBuilder::new()`] for documentation.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(args: impl IntoIterator<Item = String>) -> Result<Self, RclrsError> {
        Self::builder(args).build()
    }

    /// Checks if the context is still valid.
    ///
    /// This will return `false` when a signal has caused the context to shut down (currently
    /// unimplemented).
    pub fn ok(&self) -> bool {
        // This will currently always return true, but once we have a signal handler, the signal
        // handler could call `rcl_shutdown()`, hence making the context invalid.
        let rcl_context = &mut *self.rcl_context_mtx.lock().unwrap();
        // SAFETY: No preconditions for this function.
        unsafe { rcl_context_is_valid(rcl_context) }
    }

    /// Returns the context domain id.
    ///
    /// The domain ID controls which nodes can send messages to each other, see the [ROS 2 concept article][1].
    /// It can be set through the `ROS_DOMAIN_ID` environment variable
    /// or [`ContextBuilder`][2]
    ///
    /// [1]: https://docs.ros.org/en/rolling/Concepts/About-Domain-ID.html
    /// [2]: crate::ContextBuilder
    ///
    /// # Example
    /// ```
    /// # use rclrs::{Context, RclrsError};
    /// // Set default ROS domain ID to 10 here
    /// std::env::set_var("ROS_DOMAIN_ID", "10");
    /// let context = Context::new([])?;
    /// let domain_id = context.domain_id();
    /// assert_eq!(domain_id, 10);
    /// // Set ROS domain ID by builder
    /// let context = Context::builder([]).domain_id(11).build()?;
    /// let domain_id = context.domain_id();
    /// assert_eq!(domain_id, 11);
    /// # Ok::<(), RclrsError>(())
    /// ```
    #[cfg(not(ros_distro = "foxy"))]
    pub fn domain_id(&self) -> usize {
        let mut domain_id: usize = 0;

        let ret = unsafe {
            let mut rcl_context = self.rcl_context_mtx.lock().unwrap();
            // SAFETY: No preconditions for this function.
            rcl_context_get_domain_id(&mut *rcl_context, &mut domain_id)
        };

        debug_assert_eq!(ret, 0);
        domain_id
    }

    /// Returns the context domain id.
    ///
    /// The domain ID controls which nodes can send messages to each other, see the [ROS 2 concept article][1].
    /// It can be set through the `ROS_DOMAIN_ID` environment variable
    /// or [`ContextBuilder`][2]
    ///
    /// [1]: https://docs.ros.org/en/rolling/Concepts/About-Domain-ID.html
    /// [2]: crate::ContextBuilder
    ///
    /// # Example
    /// ```
    /// # use rclrs::{Context, RclrsError};
    /// // Set default ROS domain ID to 10 here
    /// std::env::set_var("ROS_DOMAIN_ID", "10");
    /// let context = Context::new([])?;
    /// let domain_id = context.domain_id();
    /// assert_eq!(domain_id, 10);
    /// # Ok::<(), RclrsError>(())
    /// ```
    #[cfg(ros_distro = "foxy")]
    pub fn domain_id(&self) -> usize {
        let mut domain_id: usize = 0;

        let ret = unsafe {
            // SAFETY: Getting the default domain ID, based on the environment
            rcl_get_default_domain_id(&mut domain_id)
        };

        debug_assert_eq!(ret, 0);
        domain_id
    }

    /// Creates a [`ContextBuilder`][1] with the given name.
    ///
    /// Convenience function equivalent to [`ContextBuilder::new()`][2].
    ///
    /// [1]: crate::ContextBuilder
    /// [2]: crate::ContextBuilder::new
    ///
    /// # Example
    /// ```
    /// # use rclrs::{Context, RclrsError};
    /// let mut context_builder = Context::builder([]);
    /// assert!(context_builder.build().is_ok());
    /// # Ok::<(), RclrsError>(())
    /// ```
    pub fn builder(args: impl IntoIterator<Item = String>) -> ContextBuilder {
        ContextBuilder::new(args)
    }

    /// Add on shutdown callback
    ///
    /// To trigger wait set, this callback will be invoked after context shutdown.
    pub fn add_on_shutdown_callback(&mut self, callback: Box<dyn Fn() + Send + Sync>) {
        self.shutdown_callback = Some(callback);
    }

    /// Shutdown the context, making it uninitialized and therefore invalid for derived entities.
    ///
    /// # Example
    /// ```
    /// # use rclrs::{Context, RclrsError};
    /// let context = Context::new([])?;
    /// assert!(context.shutdown());
    /// ```
    pub fn shutdown(&self) -> bool {
        unsafe {
            let rcl_context = &mut *self.rcl_context_mtx.lock().unwrap();
            // The context may be invalid when rcl_init failed, e.g. because of invalid command
            // line arguments.
            // SAFETY: No preconditions for this function.
            if !rcl_context_is_valid(rcl_context) {
                return false;
            }
            // SAFETY: These functions have no preconditions besides a valid rcl_context
            let ret = rcl_shutdown(rcl_context);
            if let Err(e) = to_rclrs_result(ret) {
                panic!("Failed to finalize context: {:?}", e);
            }

            if let Some(callback) = &self.shutdown_callback {
                callback();
            }

            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    #[test]
    fn context_is_send_and_sync() {
        assert_send::<Context>();
        assert_sync::<Context>();
    }

    #[test]
    fn test_create_context() -> Result<(), RclrsError> {
        // If the context fails to be created, this will cause a panic
        let _ = Context::new(vec![])?;
        Ok(())
    }

    #[test]
    fn test_context_ok() -> Result<(), RclrsError> {
        // If the context fails to be created, this will cause a panic
        let created_context = Context::new(vec![]).unwrap();
        assert!(created_context.ok());

        Ok(())
    }
}
