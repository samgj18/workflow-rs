// 1. User calls `workflow` from terminal and passes in a workflow name
// 2. `workflow` calls `workflow::run` with the workflow name
// 3. `workflow::run` calls `workflow::get_workflow` with the workflow name
// 4. `workflow::get_workflow` returns a `Workflow` struct
// 5. `workflow::run` calls `workflow::run_workflow` with the `Workflow` struct
// 6. `workflow::run_workflow` runs the workflow
// 7. `workflow::run_workflow` returns a `Result` with the workflow result
// 8. `workflow::run` prints the workflow result
// 9. `workflow` exits
pub mod domain;
