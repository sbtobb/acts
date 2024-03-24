mod block;
mod call;
mod cmd;
#[cfg(feature = "wit")]
mod pack;
mod req;

use super::TaskLifeCycle;
use crate::{
    event::ActionState,
    sch::Context,
    utils::{self, consts},
    Act, ActTask, Block, Result, Vars,
};
use async_trait::async_trait;
use std::{cell::RefCell, rc::Rc};

#[async_trait]
impl ActTask for Act {
    fn init(&self, ctx: &Context) -> Result<()> {
        match self {
            Act::Req(req) => req.init(ctx),
            Act::Call(u) => u.init(ctx),
            Act::Block(b) => b.init(ctx),

            #[cfg(feature = "wit")]
            Act::Pack(p) => p.init(ctx),
            _ => Ok(()),
        }
    }

    fn run(&self, ctx: &Context) -> Result<()> {
        match self {
            Act::Req(req) => req.run(ctx),
            Act::Call(u) => u.run(ctx),
            Act::Block(b) => b.run(ctx),
            #[cfg(feature = "wit")]
            Act::Pack(p) => p.run(ctx),
            _ => Ok(()),
        }
    }

    fn next(&self, ctx: &Context) -> Result<bool> {
        match self {
            Act::Req(req) => req.next(ctx),
            Act::Call(u) => u.next(ctx),
            Act::Block(b) => b.next(ctx),
            #[cfg(feature = "wit")]
            Act::Pack(p) => p.next(ctx),
            _ => Ok(false),
        }
    }

    fn review(&self, ctx: &Context) -> Result<bool> {
        match self {
            Act::Req(req) => req.review(ctx),
            Act::Call(u) => u.review(ctx),
            Act::Block(b) => b.review(ctx),
            #[cfg(feature = "wit")]
            Act::Pack(p) => p.review(ctx),
            _ => Ok(true),
        }
    }
}

impl Act {
    pub fn exec(&self, ctx: &Context) -> Result<()> {
        match self {
            Act::Set(vars) => {
                let inputs = utils::fill_inputs(&ctx.task.env(), vars);
                ctx.task.env().set_env(&inputs);
            }
            Act::Expose(vars) => {
                let outputs = utils::fill_outputs(&ctx.task.env(), vars);
                if let Some(task) = ctx.task.parent() {
                    task.env().set_env(&outputs);
                } else {
                    ctx.task.env().set(consts::ACT_OUTPUTS, outputs);
                }
            }
            Act::Req(req) => {
                let mut req = req.clone();
                if let Some(v) = ctx.get_var::<u32>(consts::ACT_INDEX) {
                    req.inputs.set(consts::ACT_INDEX, v);
                }

                if let Some(v) = ctx.get_var::<String>(consts::ACT_VALUE) {
                    req.inputs.set(consts::ACT_VALUE, v);
                }
                ctx.append_act(&Act::Req(req))?;
            }
            Act::Msg(msg) => {
                let mut msg = msg.clone();
                if let Some(v) = ctx.get_var::<u32>(consts::ACT_INDEX) {
                    msg.inputs.set(consts::ACT_INDEX, v);
                }

                if let Some(v) = ctx.get_var::<String>(consts::ACT_VALUE) {
                    msg.inputs.set(consts::ACT_VALUE, v);
                }
                if ctx.task.state().is_none() {
                    ctx.task
                        .add_hook_stmts(TaskLifeCycle::Created, &Act::Msg(msg.clone()));
                } else {
                    ctx.emit_message(&msg);
                }
            }
            Act::Cmd(cmd) => {
                if ctx.task.state().is_none() {
                    ctx.task.add_hook_stmts(TaskLifeCycle::Created, self);
                } else {
                    if let Err(err) = cmd.run(ctx) {
                        ctx.task.set_pure_action_state(ActionState::Error);
                        return Err(err);
                    };
                }
            }
            Act::Block(b) => {
                ctx.append_act(&Act::Block(b.clone()))?;
            }
            #[cfg(feature = "wit")]
            Act::Pack(p) => {
                ctx.append_act(&Act::Pack(p.clone()))?;
            }
            Act::If(cond) => {
                let result = ctx.eval(&cond.on)?;
                if result {
                    for s in &cond.then {
                        s.exec(ctx)?;
                    }
                } else {
                    for s in &cond.r#else {
                        s.exec(ctx)?;
                    }
                }
            }
            Act::Each(each) => {
                let can = each.parse(ctx, &each.r#in)?;
                for (index, value) in can.values()?.iter().enumerate() {
                    ctx.set_var(consts::ACT_INDEX, index);
                    ctx.set_var(consts::ACT_VALUE, value);
                    for s in &each.run {
                        s.exec(ctx)?;
                    }
                }
            }
            Act::Chain(chain) => {
                let cans = chain.parse(ctx, &chain.r#in)?;
                let stmts = &chain.run;
                let cands = cans.values()?;
                let mut items = cands.iter().enumerate();
                if let Some((index, value)) = items.next() {
                    let head = Rc::new(RefCell::new(Block::new()));

                    head.borrow_mut().id = utils::shortid();
                    head.borrow_mut().inputs = Vars::new()
                        .with(consts::ACT_INDEX, index)
                        .with(consts::ACT_VALUE, value);
                    head.borrow_mut().acts = stmts.clone();

                    let mut pre = head.clone();
                    while let Some((index, value)) = items.next() {
                        let p = Rc::new(RefCell::new(Block::new()));
                        p.borrow_mut().id = utils::shortid();
                        p.borrow_mut().inputs = Vars::new()
                            .with(consts::ACT_INDEX, index)
                            .with(consts::ACT_VALUE, value);
                        p.borrow_mut().acts = stmts.clone();

                        pre.borrow_mut().next = Some(Box::new((*p).clone().into_inner()));
                        pre = p;
                    }

                    let act = Act::Block(head.take());
                    act.exec(ctx)?;
                }
            }
            Act::Call(u) => {
                ctx.append_act(&Act::Call(u.clone()))?;
            }
            Act::OnCreated(stmts) => {
                for s in stmts {
                    ctx.task.add_hook_stmts(TaskLifeCycle::Created, s);
                }
            }
            Act::OnCompleted(stmts) => {
                for s in stmts {
                    ctx.task.add_hook_stmts(TaskLifeCycle::Completed, s);
                }
            }
            Act::OnBeforeUpdate(stmts) => {
                for s in stmts {
                    ctx.task.add_hook_stmts(TaskLifeCycle::BeforeUpdate, s);
                }
            }
            Act::OnUpdated(stmts) => {
                for s in stmts {
                    ctx.task.add_hook_stmts(TaskLifeCycle::Updated, s);
                }
            }
            Act::OnStep(stmts) => {
                for s in stmts {
                    ctx.task.add_hook_stmts(TaskLifeCycle::Step, s);
                }
            }
            Act::OnErrorCatch(stmts) => {
                for s in stmts {
                    ctx.task.add_hook_catch(TaskLifeCycle::ErrorCatch, s);
                }
            }
            Act::OnTimeout(stmts) => {
                for s in stmts {
                    ctx.task.add_hook_timeout(TaskLifeCycle::Timeout, s);
                }
            }
        }
        Ok(())
    }
}
