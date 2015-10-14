/*!


*/

use backend::Facade;
use context::Context;
use ContextExt;
use version::Api;
use version::Version;
use gl;
use std::rc::Rc;
use std::mem;

pub use context::DebugCallbackBehavior;

/// Represents a callback that can be used for the debug output feature of OpenGL.
///
/// The first three parameters are self-explanatory. The fourth parameter is an identifier for this
/// message whose meaning is implementation-defined. The fifth parameter indicates whether glium
/// is already handling any possible error condition, so you don't need to print an error. The last
/// parameter is a message generated by the OpenGL implementation.
pub type DebugCallback = Box<FnMut(Source, MessageType, Severity, u32, bool, &str)>;

/// Severity of a debug message.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Severity {
    /// Anything that isn't an error or performance issue.
    Notification = gl::DEBUG_SEVERITY_NOTIFICATION,

    /// Redundant state-change performance warning, or unimportant undefined behavior.
    Low = gl::DEBUG_SEVERITY_LOW,

    /// Major performance warnings, shader compilation/linking warnings,
    /// or the use of deprecated functionality.
    Medium = gl::DEBUG_SEVERITY_MEDIUM,

    /// All OpenGL Errors, shader compilation/linking errors,
    /// or highly-dangerous undefined behavior.
    High = gl::DEBUG_SEVERITY_HIGH,
}

/// Source of a debug message.
#[derive(Clone, Copy, Debug)]
#[repr(u32)]
pub enum Source {
    /// Calls to the OpenGL API.
    Api = gl::DEBUG_SOURCE_API,

    /// Calls to a window-system API.
    WindowSystem = gl::DEBUG_SOURCE_WINDOW_SYSTEM,

    /// A compiler for a shading language.
    ShaderCompiler = gl::DEBUG_SOURCE_SHADER_COMPILER,

    /// An application associated with Openctxt.gl.
    ThirdParty = gl::DEBUG_SOURCE_THIRD_PARTY,

    /// Explicitly generated by Glium or the application.
    ///
    /// This should never happen, but is included here for completeness.
    Application = gl::DEBUG_SOURCE_APPLICATION,

    ///
    OtherSource = gl::DEBUG_SOURCE_OTHER,
}

/// Type of a debug message.
#[derive(Clone, Copy, Debug)]
#[repr(u32)]
pub enum MessageType {
    /// An error, typically from the API
    Error = gl::DEBUG_TYPE_ERROR,
    /// Some behavior marked deprecated has been used
    DeprecatedBehavior = gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR,
    /// Something has invoked undefined behavior
    UndefinedBehavior = gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR,
    /// Some functionality the user relies upon is not portable
    Portability = gl::DEBUG_TYPE_PORTABILITY,
    /// Code has triggered possible performance issues
    Performance = gl::DEBUG_TYPE_PERFORMANCE,
    /// Command stream annotation
    Marker = gl::DEBUG_TYPE_MARKER,
    /// Entering a debug group
    PushGroup = gl::DEBUG_TYPE_PUSH_GROUP,
    /// Leaving a debug group
    PopGroup = gl::DEBUG_TYPE_POP_GROUP,
    /// Any other event
    Other = gl::DEBUG_TYPE_OTHER,
}

/// Allows you to obtain the timestamp inside the OpenGL commands queue.
///
/// When you call functions in glium, they are not instantly executed. Instead they are
/// added in a commands queue that the backend executes asynchronously.
///
/// When you call `TimestampQuery::new`, a command is added to this list asking the
/// backend to send us the current timestamp. Thanks to this, you can know how much time
/// it takes to execute commands.
///
/// ## Example
///
/// ```no_run
/// # let display: glium::Display = unsafe { std::mem::uninitialized() };
/// let before = glium::debug::TimestampQuery::new(&display);
/// // do some stuff here
/// let after = glium::debug::TimestampQuery::new(&display);
///
/// match (after, before) {
///     (Some(after), Some(before)) => {
///         let elapsed = after.get() - before.get();
///         println!("Time it took to do stuff: {}", elapsed);
///     },
///     _ => ()
/// }
/// ```
///
pub struct TimestampQuery {
    context: Rc<Context>,
    id: gl::types::GLuint,
}

impl TimestampQuery {
    /// Creates a new `TimestampQuery`. Returns `None` if the backend doesn't support it.
    pub fn new<F>(facade: &F) -> Option<TimestampQuery> where F: Facade {
        let ctxt = facade.get_context().make_current();

        let id = if ctxt.version >= &Version(Api::Gl, 3, 2) {    // TODO: extension
            unsafe {
                let mut id = mem::uninitialized();
                ctxt.gl.GenQueries(1, &mut id);

                ctxt.gl.QueryCounter(id, gl::TIMESTAMP);

                Some(id)
            }

        } else if ctxt.extensions.gl_ext_disjoint_timer_query {
            unsafe {
                let mut id = mem::uninitialized();
                ctxt.gl.GenQueriesEXT(1, &mut id);

                ctxt.gl.QueryCounterEXT(id, gl::TIMESTAMP);

                Some(id)
            }

        } else {
            None
        };

        id.map(|q| TimestampQuery {
            context: facade.get_context().clone(),
            id: q
        })
    }

    /// Queries the counter to see if the timestamp is already available.
    ///
    /// It takes some time to retreive the value, during which you can execute other
    /// functions.
    pub fn is_ready(&self) -> bool {
        use std::mem;

        let ctxt = self.context.make_current();

        if ctxt.version >= &Version(Api::Gl, 3, 2) {    // TODO: extension
            unsafe {
                let mut value = mem::uninitialized();
                ctxt.gl.GetQueryObjectiv(self.id, gl::QUERY_RESULT_AVAILABLE, &mut value);
                value != 0
            }

        } else if ctxt.extensions.gl_ext_disjoint_timer_query {
            unsafe {
                let mut value = mem::uninitialized();
                ctxt.gl.GetQueryObjectivEXT(self.id, gl::QUERY_RESULT_AVAILABLE_EXT, &mut value);
                value != 0
            }

        } else {
            unreachable!();
        }
    }

    /// Returns the value of the timestamp. Blocks until it is available.
    ///
    /// This function doesn't block if `is_ready` returns true.
    pub fn get(self) -> u64 {
        use std::mem;

        let ctxt = self.context.make_current();

        if ctxt.version >= &Version(Api::Gl, 3, 2) {    // TODO: extension
            unsafe {
                let mut value = mem::uninitialized();
                ctxt.gl.GetQueryObjectui64v(self.id, gl::QUERY_RESULT, &mut value);
                ctxt.gl.DeleteQueries(1, [self.id].as_ptr());
                value
            }

        } else if ctxt.extensions.gl_ext_disjoint_timer_query {
            unsafe {
                let mut value = mem::uninitialized();
                ctxt.gl.GetQueryObjectui64vEXT(self.id, gl::QUERY_RESULT_EXT, &mut value);
                ctxt.gl.DeleteQueriesEXT(1, [self.id].as_ptr());
                value
            }

        } else {
            unreachable!();
        }
    }
}
