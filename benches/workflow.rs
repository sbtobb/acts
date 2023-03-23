use acts::{ActionOptions, Engine, State, Workflow};
use criterion::*;
use tokio::runtime::Runtime;

fn simple_workflow(c: &mut Criterion) {
    let text = r#"
  name: test1
  ver: 1.0
  env: 
    a: 100
  jobs:
    - id: job1
      steps:
        - name: step 1
          run: |
            //print("step 1")
        - name: step 2
          run: |
            //print("step 2");
            let v = 50;
            //console::log(`v=${v}`);
            //console::dbg(`v=${v}`);
            //console::info(`v=${v}`);
            //console::wran(`v=${v}`);
            //console::error(`v=${v}`);
        - name: step 3
          env:
            e: abc
          branches:
            - name: branch 1
              if: env.get("a") >= 100
              steps:
                - name: branch 1.1
                  run: |
                    //print("branch 1.1");
                - name: branch 1.2
                  run: // print("branch 1.2")
            - name: branch 2
              if: env.get("a") < 100
              steps:
                - name:  branch 2.1
                  run: // print("branch 2.1")
          run: |
            // print("step 3");

        - name: step 4
          run: 
            // print(`step 4`);
  
  "#;
    c.bench_function("simple_workflow", |b| {
        let rt = Runtime::new().unwrap();
        let engine = Engine::new();
        engine.start();
        let workflow = Workflow::from_str(text).unwrap();
        let executor = engine.executor();
        let e = engine.clone();
        executor.deploy(&workflow).unwrap();

        engine.emitter().on_complete(move |_w: &State<Workflow>| {
            //e.close();
        });

        b.iter(move || {
            let workflow = Workflow::from_str(text).unwrap();
            let exec = e.executor();
            rt.block_on(async move {
                exec.start(
                    &workflow.id,
                    ActionOptions {
                        biz_id: Some("w1".into()),
                        ..Default::default()
                    },
                )
                .unwrap();
            });
        })
    });
}

criterion_group!(benches, simple_workflow);
criterion_main!(benches);
