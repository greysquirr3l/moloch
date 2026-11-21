#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(input) = std::str::from_utf8(data) else {
        return;
    };
    let Ok(plan) = moloch::domain::plan::Plan::from_toml(input) else {
        return;
    };
    let tasks = plan.resolve_tasks().unwrap_or_default();

    let _ = moloch::generation::agents_md::render(&plan);
    let _ = moloch::generation::planner::render(&plan);
    let _ = moloch::generation::background_auditor::render(&plan);

    if let Some(task) = tasks.first() {
        let _ = moloch::generation::task::render(&plan, task);
    }

    let _ = moloch::generation::orchestrator::render(&plan, &tasks);
    let _ = moloch::generation::progress::render(&plan, &tasks);
    let _ = moloch::generation::plan_doc::render(&plan, &tasks);
    let _ = moloch::generation::features::render(&plan, &tasks);
    let _ = moloch::generation::evaluator::render(&plan, &tasks);
});
