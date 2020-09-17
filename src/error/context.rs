use core::any::Any;
use core::fmt::{self, Debug};

#[cfg(feature = "full-context")]
use alloc::{boxed::Box, vec::Vec};

/// The base context surrounding an error.
pub trait Context: Any + Debug {
    /// The operation that was attempted when an error occured.
    ///
    /// It should described in a simple manner what is trying to be achieved and
    /// make sense in the following sentence if you were to substitute it:
    ///
    /// ```text
    /// Something failed while attempting to <operation> from the input.
    /// ```
    fn operation(&self) -> &'static str;

    /// Returns a [`fmt::Display`] formattable value of what was expected.
    fn expected(&self) -> Option<&dyn fmt::Display>;
}

/// A walkable stack of contexts collected from an error.
pub trait ContextStack {
    /// The root context.
    fn root(&self) -> ExpectedContext;

    /// Walk the context stack, starting with the highest context to the root.
    ///
    /// Returns `true` if all of the stack available was walked, `false` if not.
    fn walk<'a>(&'a self, f: &mut ContextStackWalker<'a>) -> bool;
}

/// A [`ContextStack`] builder.
pub trait ContextStackBuilder {
    /// Create the builder from a root expected context.
    fn from_root(context: ExpectedContext) -> Self;

    /// Push an additional context onto the stack.
    fn push<C>(&mut self, context: C)
    where
        C: Context;
}

/// A dynamic function for walking a context stack.
///
/// Returns `true` if the walk should continue, `false` if not.
///
/// # Parameters
///
/// - `index` (the index of the context starting from `1`).
/// - `context` (the context at the provided index).
pub type ContextStackWalker<'a> = dyn FnMut(usize, &'a dyn Context) -> bool + 'a;

///////////////////////////////////////////////////////////////////////////////
// Basic expected context

impl Context for &'static str {
    fn operation(&self) -> &'static str {
        "read"
    }

    fn expected(&self) -> Option<&dyn fmt::Display> {
        Some(self)
    }
}

///////////////////////////////////////////////////////////////////////////////
// Expected context

/// A sealed expected context.
#[derive(Clone, Copy, Debug)]
pub struct ExpectedContext {
    pub(crate) operation: &'static str,
    pub(crate) expected: &'static str,
}

impl Context for ExpectedContext {
    fn operation(&self) -> &'static str {
        self.operation
    }

    fn expected(&self) -> Option<&dyn fmt::Display> {
        Some(&self.expected)
    }
}

///////////////////////////////////////////////////////////////////////////////
// Operation context

#[derive(Clone, Copy, Debug)]
pub(crate) struct OperationContext(pub(crate) &'static str);

impl Context for OperationContext {
    fn operation(&self) -> &'static str {
        self.0
    }

    fn expected(&self) -> Option<&dyn fmt::Display> {
        None
    }
}

///////////////////////////////////////////////////////////////////////////////
// Root context stack

/// A context stack that only contains the root [`ExpectedContext`].
pub struct RootContextStack {
    context: ExpectedContext,
}

impl ContextStackBuilder for RootContextStack {
    fn from_root(context: ExpectedContext) -> Self {
        Self { context }
    }

    fn push<C>(&mut self, _context: C)
    where
        C: Context,
    {
    }
}

impl ContextStack for RootContextStack {
    fn root(&self) -> ExpectedContext {
        self.context
    }

    fn walk<'a>(&'a self, f: &mut ContextStackWalker<'a>) -> bool {
        f(1, &self.context)
    }
}

///////////////////////////////////////////////////////////////////////////////
// Full context stack

/// A context stack that contains all contexts collected.
#[cfg(feature = "full-context")]
#[cfg_attr(docsrs, doc(cfg(feature = "full-context")))]
pub struct FullContextStack {
    root: ExpectedContext,
    stack: Vec<Box<dyn Context>>,
}

#[cfg(feature = "full-context")]
impl ContextStackBuilder for FullContextStack {
    fn from_root(context: ExpectedContext) -> Self {
        Self {
            root: context,
            stack: Vec::with_capacity(32),
        }
    }

    fn push<C>(&mut self, context: C)
    where
        C: Context,
    {
        self.stack.push(Box::new(context))
    }
}

#[cfg(feature = "full-context")]
impl ContextStack for FullContextStack {
    fn root(&self) -> ExpectedContext {
        self.root
    }

    fn walk<'a>(&'a self, f: &mut ContextStackWalker<'a>) -> bool {
        let mut i = 1;
        for item in self.stack.iter().rev() {
            if !f(i, item.as_ref()) {
                return false;
            }
            i += 1;
        }
        f(i, &self.root)
    }
}
