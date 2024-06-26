mod context;
mod proc;
mod queue;
mod scher;
mod state;
mod tree;

#[cfg(test)]
mod tests;

use async_trait::async_trait;

use crate::event::ActionState;
pub use crate::Result;
pub use context::Context;
pub use proc::{Proc, StatementBatch, Task, TaskLifeCycle};
pub use scher::Scheduler;
pub use state::TaskState;
pub use tree::{Node, NodeContent, NodeKind, NodeTree};

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
        Ok(true)
    }

    fn error(&self, ctx: &Context) -> Result<()> {
        if !ctx.task.state().is_error() {
            let err = ctx.err().unwrap_or_default();
            ctx.task.set_pure_action_state(ActionState::Error);
            ctx.task.set_state(TaskState::Fail(err.to_string()));
        }
        ctx.emit_error()
    }
}
