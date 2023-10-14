use crate::{
    event::{ActionState, Emitter},
    sch::{Proc, Scheduler},
    utils, Action, Engine, Vars, Workflow,
};
use serde_json::json;
use std::sync::{Arc, Mutex};

#[tokio::test]
async fn sch_act_for_rule_empty() {
    let mut workflow = Workflow::new().with_job(|job| {
        job.with_id("job1").with_step(|step| {
            step.with_id("step1")
                .with_act(|act| act.with_for(|f| f.with_in(r#"["u1"]"#)))
        })
    });
    workflow.print();
    let (proc, scher, emitter) = create_proc(&mut workflow, &utils::longid());

    let ret = Arc::new(Mutex::new(false));
    let ret2 = ret.clone();
    emitter.on_error(move |_| {
        // the error is expected
        *ret.lock().unwrap() = true;
    });
    scher.launch(&proc);
    scher.event_loop().await;
    let v = ret2.lock().unwrap();
    assert_eq!(*v, true)
}

#[tokio::test]
async fn sch_act_for_rule_error() {
    let mut workflow = Workflow::new().with_job(|job| {
        job.with_id("job1").with_step(|step| {
            step.with_id("step1")
                .with_act(|act| act.with_for(|f| f.with_by("no_exist").with_in(r#"["u1"]"#)))
        })
    });
    let id = utils::longid();
    let (proc, scher, emitter) = create_proc(&mut workflow, &id);

    let ret = Arc::new(Mutex::new(false));
    let ret2 = ret.clone();
    emitter.on_error(move |_| {
        // the error is expected
        *ret.lock().unwrap() = true;
    });
    scher.launch(&proc);
    scher.event_loop().await;
    let v = ret2.lock().unwrap();
    assert_eq!(*v, true)
}

#[tokio::test]
async fn sch_act_for_rule_some_no_key_error() {
    let mut workflow = Workflow::new().with_job(|job| {
        job.with_id("job1").with_step(|step| {
            step.with_id("step1")
                .with_act(|act| act.with_for(|f| f.with_by("some").with_in(r#"["u1"]"#)))
        })
    });
    let id = utils::longid();
    let (proc, scher, emitter) = create_proc(&mut workflow, &id);

    let ret = Arc::new(Mutex::new(false));
    let ret2 = ret.clone();
    emitter.on_error(move |_| {
        // the error is expected
        *ret.lock().unwrap() = true;
    });
    scher.launch(&proc);
    scher.event_loop().await;
    let v = ret2.lock().unwrap();
    assert_eq!(*v, true)
}

#[tokio::test]
async fn sch_act_for_no_in_error() {
    let mut workflow = Workflow::new().with_job(|job| {
        job.with_id("job1").with_step(|step| {
            step.with_id("step1")
                .with_act(|act| act.with_for(|f| f.with_by("all")))
        })
    });
    let id = utils::longid();
    let (proc, scher, emitter) = create_proc(&mut workflow, &id);

    let ret = Arc::new(Mutex::new(false));
    let ret2 = ret.clone();
    emitter.on_error(move |_| {
        // the error is expected
        *ret.lock().unwrap() = true;
    });
    scher.launch(&proc);
    scher.event_loop().await;
    let v = ret2.lock().unwrap();
    assert_eq!(*v, true)
}

#[tokio::test]
async fn sch_act_for_in_empty_error() {
    let mut workflow = Workflow::new().with_job(|job| {
        job.with_id("job1").with_step(|step| {
            step.with_id("step1")
                .with_act(|act| act.with_for(|f| f.with_by("all").with_in("")))
        })
    });
    let id = utils::longid();
    let (proc, scher, emitter) = create_proc(&mut workflow, &id);

    let ret = Arc::new(Mutex::new(false));
    let ret2 = ret.clone();
    emitter.on_error(move |_| {
        // the error is expected
        *ret.lock().unwrap() = true;
    });
    scher.launch(&proc);
    scher.event_loop().await;
    let v = ret2.lock().unwrap();
    assert_eq!(*v, true)
}

#[tokio::test]
async fn sch_act_for_tag_default() {
    let ret = Arc::new(Mutex::new(false));
    let mut workflow = Workflow::new().with_job(|job| {
        job.with_id("job1").with_step(|step| {
            step.with_id("step1")
                .with_act(|act| act.with_for(|f| f.with_by("all").with_in(r#"["u1", "u2"]"#)))
        })
    });
    let id = utils::longid();
    let (proc, scher, emitter) = create_proc(&mut workflow, &id);
    let s = scher.clone();
    let r = ret.clone();
    emitter.on_message(move |msg| {
        if msg.inner().r#type == "act"
            && msg.inner().state() == ActionState::Created
            && msg.inner().tag == "for"
        {
            *r.lock().unwrap() = true;
            s.close();
        }
    });
    scher.launch(&proc);
    scher.event_loop().await;
    assert!(*ret.lock().unwrap());
}

#[tokio::test]
async fn sch_act_for_tag_key() {
    let ret = Arc::new(Mutex::new(false));
    let mut workflow = Workflow::new().with_job(|job| {
        job.with_id("job1").with_step(|step| {
            step.with_id("step1").with_act(|act| {
                act.with_tag("tag1")
                    .with_for(|f| f.with_by("all").with_in(r#"["u1", "u2"]"#))
            })
        })
    });
    let id = utils::longid();
    let (proc, scher, emitter) = create_proc(&mut workflow, &id);
    let s = scher.clone();
    let r = ret.clone();
    emitter.on_message(move |msg| {
        if msg.inner().r#type == "act"
            && msg.inner().state() == ActionState::Created
            && msg.inner().tag == "tag1"
        {
            *r.lock().unwrap() = true;
            s.close();
        }
    });
    scher.launch(&proc);
    scher.event_loop().await;
    assert_eq!(*ret.lock().unwrap(), true);
}

#[tokio::test]
async fn sch_act_for_each_default() {
    let mut workflow = Workflow::new().with_job(|job| {
        job.with_id("job1").with_step(|step| {
            step.with_id("step1")
                .with_act(|act| act.with_for(|f| f.with_by("all").with_in(r#"["u1", "u2"]"#)))
        })
    });
    let id = utils::longid();
    let (proc, scher, emitter) = create_proc(&mut workflow, &id);

    let count = Arc::new(Mutex::new(0));
    let s = scher.clone();
    emitter.on_message(move |msg| {
        if msg.inner().r#type == "act"
            && msg.inner().state() == ActionState::Created
            && msg.inner().key == "each"
        {
            let mut count = count.lock().unwrap();
            *count += 1;
            if *count == 2 {
                assert!(true);
                s.close();
            }
        }
    });
    scher.launch(&proc);
    scher.event_loop().await;
}

#[tokio::test]
async fn sch_act_for_each_key() {
    let mut workflow = Workflow::new().with_job(|job| {
        job.with_id("job1").with_step(|step| {
            step.with_id("step1").with_act(|act| {
                act.with_for(|f| {
                    f.with_by("all")
                        .with_alias(|a| a.with_each("my_each"))
                        .with_in(r#"["u1", "u2"]"#)
                })
            })
        })
    });
    let id = utils::longid();
    let (proc, scher, emitter) = create_proc(&mut workflow, &id);

    let count = Arc::new(Mutex::new(0));
    let s = scher.clone();
    emitter.on_message(move |msg| {
        if msg.inner().r#type == "act"
            && msg.inner().state() == ActionState::Created
            && msg.inner().key == "my_each"
        {
            let mut count = count.lock().unwrap();
            *count += 1;
            if *count == 2 {
                assert!(true);
                s.close();
            }
        }
    });
    scher.launch(&proc);
    scher.event_loop().await;
}

#[tokio::test]
async fn sch_act_for_init_default() {
    let mut workflow = Workflow::new().with_job(|job| {
        job.with_id("job1").with_step(|step| {
            step.with_id("step1").with_act(|act| {
                act.with_for(|f| {
                    f.with_by("all")
                        .with_in(r#"act.role("role1").union(act.unit("unit1"))"#)
                })
            })
        })
    });
    let id = utils::longid();
    let (proc, scher, emitter) = create_proc(&mut workflow, &id);

    let s = scher.clone();
    emitter.on_message(move |msg| {
        if msg.inner().r#type == "act"
            && msg.inner().state() == ActionState::Created
            && msg.inner().key == "ctor"
        {
            assert_eq!(msg.inner().inputs.get("users").is_some(), true);
            s.close();
        }
    });
    scher.launch(&proc);
    scher.event_loop().await;
}

#[tokio::test]
async fn sch_act_for_init_key() {
    let mut workflow = Workflow::new().with_job(|job| {
        job.with_id("job1").with_step(|step| {
            step.with_id("step1").with_act(|act| {
                act.with_for(|f| {
                    f.with_by("all")
                        .with_alias(|a| a.with_init("my_init"))
                        .with_in(r#"act.role("role1")"#)
                })
            })
        })
    });
    let id = utils::longid();
    let (proc, scher, emitter) = create_proc(&mut workflow, &id);
    let s = scher.clone();
    emitter.on_message(move |msg| {
        if msg.inner().r#type == "act"
            && msg.inner().state() == ActionState::Created
            && msg.inner().key == "my_init"
        {
            assert_eq!(msg.inner().inputs.get("users").is_some(), true);
            let inits = msg
                .inner()
                .inputs
                .get("users")
                .unwrap()
                .as_object()
                .unwrap();
            assert_eq!(inits.get("role(role1)").is_some(), true);
            s.close();
        }
    });
    scher.launch(&proc);
    scher.event_loop().await;
}

#[tokio::test]
async fn sch_act_for_init_action() {
    let mut workflow = Workflow::new().with_job(|job| {
        job.with_id("job1").with_step(|step| {
            step.with_id("step1")
                .with_act(|act| act.with_for(|f| f.with_by("any").with_in(r#"act.role("role1")"#)))
        })
    });
    let id = utils::longid();
    let (proc, scher, emitter) = create_proc(&mut workflow, &id);
    let s = scher.clone();
    emitter.on_message(move |msg| {
        if msg.inner().r#type == "act"
            && msg.inner().state() == ActionState::Created
            && msg.inner().key == "ctor"
        {
            assert_eq!(msg.inner().inputs.get("users").is_some(), true);
            let mut users = msg
                .inner()
                .inputs
                .get("users")
                .unwrap()
                .as_object()
                .unwrap()
                .clone();
            assert_eq!(users.get("role(role1)").is_some(), true);

            // fill the role1 value
            users
                .entry("role(role1)")
                .and_modify(|v| *v = json!(["u1", "u2"]));

            let mut options = Vars::new();
            options.insert("users".to_string(), json!(users));
            options.insert("uid".to_string(), json!("u1"));

            let action = Action::new(&msg.inner().proc_id, &msg.inner().id, "complete", &options);
            s.do_action(&action).unwrap();
        }

        if msg.inner().r#type == "act"
            && msg.inner().state() == ActionState::Created
            && msg.inner().key == "each"
        {
            s.close();
        }
    });
    scher.launch(&proc);
    scher.event_loop().await;
}

#[tokio::test]
async fn sch_act_for_ord_default() {
    let mut workflow = Workflow::new().with_job(|job| {
        job.with_id("job1").with_step(|step| {
            step.with_id("step1")
                .with_act(|act| act.with_for(|f| f.with_by("ord").with_in(r#"["u1", "u2"]"#)))
        })
    });
    let id = utils::longid();
    let (proc, scher, emitter) = create_proc(&mut workflow, &id);

    let s = scher.clone();
    emitter.on_message(move |msg| {
        if msg.inner().r#type == "act"
            && msg.inner().state() == ActionState::Created
            && msg.inner().key == "each"
        {
            assert_eq!(msg.inner().inputs.get("uid").unwrap(), &json!("u1"));
            s.close();
        }
    });
    scher.launch(&proc);
    scher.event_loop().await;
}

#[tokio::test]
async fn sch_act_for_ord_key_act_create() {
    let mut workflow = Workflow::new().with_job(|job| {
        job.with_id("job1").with_step(|step| {
            step.with_id("step1")
                .with_act(|act| act.with_for(|f| f.with_by("ord(k1)").with_in(r#"["u1", "u2"]"#)))
        })
    });
    let id = utils::longid();
    let (proc, scher, emitter) = create_proc(&mut workflow, &id);

    let s = scher.clone();
    emitter.on_message(move |msg| {
        // check get the ord act k1
        if msg.inner().r#type == "act"
            && msg.inner().state() == ActionState::Created
            && msg.inner().key == "k1"
        {
            assert!(true);
            s.close();
        }
    });
    scher.launch(&proc);
    scher.event_loop().await;
}

#[tokio::test]
async fn sch_act_for_ord_key_act_complete() {
    let ret = Arc::new(Mutex::new(false));
    let mut workflow = Workflow::new().with_job(|job| {
        job.with_id("job1").with_step(|step| {
            step.with_id("step1")
                .with_act(|act| act.with_for(|f| f.with_by("ord(k1)").with_in(r#"["u1", "u2"]"#)))
        })
    });
    let id = utils::longid();
    let (proc, scher, emitter) = create_proc(&mut workflow, &id);

    let s = scher.clone();
    let r = ret.clone();
    emitter.on_message(move |e| {
        if e.inner().r#type == "act"
            && e.inner().state() == ActionState::Created
            && e.inner().key == "k1"
        {
            let mut cands = e
                .inner()
                .inputs
                .get("cands")
                .unwrap()
                .as_array()
                .unwrap()
                .iter()
                .map(|iter| iter.as_str().unwrap())
                .collect::<Vec<_>>();

            // reverse the cands order
            cands.reverse();

            let mut options = Vars::new();
            options.insert("uid".to_string(), json!("test1"));
            options.insert("cands".to_string(), json!(cands));
            options.insert("k1".to_string(), json!(true));

            let action = Action::new(&e.inner().proc_id, &e.inner().id, "complete", &options);
            s.do_action(&action).unwrap();
        }

        if e.inner().r#type == "act"
            && e.inner().state() == ActionState::Created
            && e.inner().key == "each"
        {
            // check the first act uid is u2 which is not the original first uid
            *r.lock().unwrap() = e.inner().inputs.get("uid").unwrap() == &json!("u2");
            s.close();
        }
    });
    scher.launch(&proc);
    scher.event_loop().await;
    proc.print();
    assert_eq!(*ret.lock().unwrap(), true);
}

#[tokio::test]
async fn sch_act_for_ord_key_act_next() {
    let mut workflow = Workflow::new().with_job(|job| {
        job.with_id("job1").with_step(|step| {
            step.with_id("step1")
                .with_act(|act| act.with_for(|f| f.with_by("ord").with_in(r#"["u1", "u2"]"#)))
        })
    });
    let id = utils::longid();
    let (proc, scher, emitter) = create_proc(&mut workflow, &id);

    let s = scher.clone();
    let count = Arc::new(Mutex::new(0));
    emitter.on_message(move |msg| {
        if msg.inner().r#type == "act"
            && msg.inner().state() == ActionState::Created
            && msg.inner().key == "k1"
        {
            let mut cands = msg
                .inner()
                .inputs
                .get("cands")
                .unwrap()
                .as_array()
                .unwrap()
                .iter()
                .map(|iter| iter.as_str().unwrap())
                .collect::<Vec<_>>();

            // reverse the cands order
            cands.reverse();
        }

        if msg.inner().r#type == "act"
            && msg.inner().state() == ActionState::Created
            && msg.inner().key == "each"
        {
            let mut count = count.lock().unwrap();

            if *count == 0 {
                assert_eq!(msg.inner().inputs.get("uid").unwrap(), &json!("u1"));

                let mut options = Vars::new();
                options.insert("uid".to_string(), json!("test1"));

                let action =
                    Action::new(&msg.inner().proc_id, &msg.inner().id, "complete", &options);
                s.do_action(&action).unwrap();
            } else {
                // the next act by ord is u2
                assert_eq!(msg.inner().inputs.get("uid").unwrap(), &json!("u2"));
                s.close();
            }
            *count += 1;
        }
    });
    scher.launch(&proc);
    scher.event_loop().await;
}

#[tokio::test]
async fn sch_act_for_any() {
    let mut workflow = Workflow::new().with_job(|job| {
        job.with_id("job1").with_step(|step| {
            step.with_id("step1")
                .with_act(|act| act.with_for(|f| f.with_by("any").with_in(r#"["u1", "u2"]"#)))
        })
    });
    let id = utils::longid();
    let (proc, scher, emitter) = create_proc(&mut workflow, &id);

    let s = scher.clone();
    emitter.on_message(move |msg| {
        if msg.inner().r#type == "act"
            && msg.inner().state() == ActionState::Created
            && msg.inner().key == "each"
        {
            assert_eq!(
                msg.inner().inputs.get("cands").unwrap(),
                &json!(["u1", "u2"])
            );
            s.close();
        }
    });
    scher.launch(&proc);
    scher.event_loop().await;
}

#[tokio::test]
async fn sch_act_for_all() {
    let mut workflow = Workflow::new().with_job(|job| {
        job.with_id("job1").with_step(|step| {
            step.with_id("step1")
                .with_act(|act| act.with_for(|f| f.with_by("all").with_in(r#"["u1", "u2"]"#)))
        })
    });
    let id = utils::longid();
    let (proc, scher, emitter) = create_proc(&mut workflow, &id);

    let s = scher.clone();
    let count = Arc::new(Mutex::new(0));
    emitter.on_message(move |msg| {
        if msg.inner().r#type == "act"
            && msg.inner().state() == ActionState::Created
            && msg.inner().key == "each"
        {
            let mut count = count.lock().unwrap();

            if *count == 0 {
                assert_eq!(msg.inner().inputs.get("uid").unwrap(), &json!("u1"));
            } else if *count == 1 {
                assert_eq!(msg.inner().inputs.get("uid").unwrap(), &json!("u2"));
                s.close();
            }

            *count += 1;
        }
    });
    scher.launch(&proc);
    scher.event_loop().await;
}

#[tokio::test]
async fn sch_act_for_some_key() {
    let mut workflow = Workflow::new().with_job(|job| {
        job.with_id("job1").with_step(|step| {
            step.with_id("step1").with_act(|act| {
                act.with_for(|f| f.with_by("some(some1)").with_in(r#"["u1", "u2", "u3"]"#))
            })
        })
    });
    let id = utils::longid();
    let (proc, scher, emitter) = create_proc(&mut workflow, &id);

    let each_count = Arc::new(Mutex::new(0));
    let some_count = Arc::new(Mutex::new(0));

    let s1 = scher.clone();
    emitter.on_complete(move |_| {
        s1.close();
    });

    let s2 = scher.clone();
    emitter.on_message(move |msg| {
        if msg.inner().r#type == "act"
            && msg.inner().state() == ActionState::Created
            && msg.inner().key == "each"
        {
            let mut count = each_count.lock().unwrap();

            // do twice action to genereate twice some1 rule acts
            if *count == 0 {
                assert_eq!(msg.inner().inputs.get("uid").unwrap(), &json!("u1"));
                let mut options = Vars::new();
                options.insert("uid".to_string(), json!("u1"));

                let action =
                    Action::new(&msg.inner().proc_id, &msg.inner().id, "complete", &options);
                s2.do_action(&action).unwrap();
            } else if *count == 1 {
                assert_eq!(msg.inner().inputs.get("uid").unwrap(), &json!("u2"));
                let mut options = Vars::new();
                options.insert("uid".to_string(), json!("u2"));

                let action =
                    Action::new(&msg.inner().proc_id, &msg.inner().id, "complete", &options);
                s2.do_action(&action).unwrap();
            }

            *count += 1;
        }

        if msg.inner().r#type == "act"
            && msg.inner().state() == ActionState::Created
            && msg.inner().key == "some1"
        {
            let mut count = some_count.lock().unwrap();
            assert!(msg.inner().inputs.get("acts").is_some());

            if *count == 2 {
                // in the second some1 act, complete it by mark the result as true
                let mut options = Vars::new();
                options.insert("some1".to_string(), json!(true));
                options.insert("uid".to_string(), json!("sys"));

                let action =
                    Action::new(&msg.inner().proc_id, &msg.inner().id, "complete", &options);
                s2.do_action(&action).unwrap();
            }

            *count += 1;
        }
    });
    scher.launch(&proc);
    scher.event_loop().await;
}

#[tokio::test]
async fn sch_act_for_many_steps() {
    let ret = Arc::new(Mutex::new(false));
    let mut workflow = Workflow::new().with_job(|job| {
        job.with_id("job1")
            .with_step(|step| {
                step.with_id("step1").with_act(|act| {
                    act.with_tag("tag1")
                        .with_for(|f| f.with_by("any").with_in(r#"["u1", "u2", "u3"]"#))
                })
            })
            .with_step(|step| {
                step.with_id("step2").with_act(|act| {
                    act.with_tag("tag2")
                        .with_for(|f| f.with_by("any").with_in(r#"["u1", "u2", "u3"]"#))
                })
            })
    });
    let id = utils::longid();
    let (proc, scher, emitter) = create_proc(&mut workflow, &id);

    let s1 = scher.clone();
    let r = ret.clone();
    emitter.on_complete(move |_| {
        *r.lock().unwrap() = true;
        s1.close();
    });

    let s2 = scher.clone();
    emitter.on_message(move |msg| {
        if msg.inner().r#type == "act"
            && msg.inner().state() == ActionState::Created
            && msg.inner().tag == "tag1"
        {
            let mut options = Vars::new();
            options.insert("uid".to_string(), json!("u1"));

            let action = Action::new(&msg.inner().proc_id, &msg.inner().id, "complete", &options);
            s2.do_action(&action).unwrap();
        }

        if msg.inner().r#type == "act"
            && msg.inner().state() == ActionState::Created
            && msg.inner().tag == "tag2"
        {
            let mut options = Vars::new();
            options.insert("uid".to_string(), json!("u2"));

            let action = Action::new(&msg.inner().proc_id, &msg.inner().id, "complete", &options);
            s2.do_action(&action).unwrap();
        }
    });
    scher.launch(&proc);
    scher.event_loop().await;
    proc.print();
    assert_eq!(*ret.lock().unwrap(), true);
}

fn create_proc(workflow: &mut Workflow, pid: &str) -> (Arc<Proc>, Arc<Scheduler>, Arc<Emitter>) {
    let engine = Engine::new();
    let scher = engine.scher();

    let proc = Arc::new(Proc::new(&pid));
    proc.load(workflow);

    let emitter = scher.emitter().clone();
    let s = scher.clone();
    emitter.on_complete(move |p| {
        if p.inner().state.is_completed() {
            s.close();
        }
    });

    let s2 = scher.clone();
    emitter.on_error(move |p| {
        println!("error in '{}', error={}", p.inner().pid, p.inner().state);
        s2.close();
    });
    (proc, scher, emitter)
}