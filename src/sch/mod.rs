mod context;
mod proc;
mod queue;
mod scher;
mod state;
mod tree;

#[cfg(test)]
mod tests;

use async_trait::async_trait;
use core::clone::Clone;

pub use crate::Result;
pub use context::Context;
pub use proc::{Proc, Task};
pub use scher::Scheduler;
pub use state::TaskState;
pub use tree::{Node, NodeData, NodeKind, NodeTree};

#[async_trait]
pub trait ActTask: Clone + Send {
    fn init(&self, _ctx: &Context) -> Result<()> {
        Ok(())
    }

    fn run(&self, _ctx: &Context) -> Result<()> {
        Ok(())
    }

    fn next(&self, _ctx: &Context) -> Result<bool> {
        Ok(false)
    }

    fn review(&self, _ctx: &Context) -> Result<bool> {
        Ok(false)
    }
}
