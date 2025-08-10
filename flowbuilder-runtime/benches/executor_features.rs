use criterion::{black_box, criterion_group, criterion_main, Criterion};
use flowbuilder_core::{
    ActionSpec, ExecutionNode, ExecutionPhase, ExecutionPlan,
    PhaseExecutionMode,
};
use flowbuilder_runtime::EnhancedTaskExecutor;
use std::collections::HashMap;
use std::sync::Arc;

fn build_plan(parallel: bool, nodes: usize) -> ExecutionPlan {
    let mut plan = ExecutionPlan::new(
        "bench".into(),
        "1".into(),
        HashMap::new(),
        HashMap::new(),
    );
    let mut phase = ExecutionPhase {
        id: "phase1".into(),
        name: "Phase1".into(),
        execution_mode: if parallel {
            PhaseExecutionMode::Parallel
        } else {
            PhaseExecutionMode::Sequential
        },
        nodes: Vec::new(),
        condition: None,
    };
    for i in 0..nodes {
        phase.nodes.push(ExecutionNode::new(
            format!("n{i}"),
            format!("Node {i}"),
            ActionSpec {
                action_type: "builtin".into(),
                parameters: HashMap::new(),
                outputs: HashMap::new(),
            },
        ));
    }
    plan.add_phase(phase);
    plan
}

fn bench_executor(c: &mut Criterion) {
    let mut group = c.benchmark_group("executor_features");

    for &nodes in &[10usize, 50] {
        // baseline: sequential
        group.bench_function(format!("seq_{nodes}"), |b| {
            b.to_async(tokio::runtime::Runtime::new().unwrap()).iter(
                || async move {
                    let mut exec = EnhancedTaskExecutor::new();
                    let plan = build_plan(false, nodes);
                    let ctx = Arc::new(tokio::sync::Mutex::new(
                        flowbuilder_context::FlowContext::default(),
                    ));
                    let _ = exec.execute_plan(plan, ctx).await.unwrap();
                },
            );
        });
        // parallel (if feature enabled)
        #[cfg(feature = "parallel")]
        group.bench_function(format!("par_{nodes}"), |b| {
            b.to_async(tokio::runtime::Runtime::new().unwrap()).iter(
                || async move {
                    let mut exec = EnhancedTaskExecutor::new();
                    let plan = build_plan(true, nodes);
                    let ctx = Arc::new(tokio::sync::Mutex::new(
                        flowbuilder_context::FlowContext::default(),
                    ));
                    let _ = exec.execute_plan(plan, ctx).await.unwrap();
                },
            );
        });
    }
    group.finish();
}

criterion_group!(name=benches; config=Criterion::default(); targets=bench_executor);
criterion_main!(benches);
