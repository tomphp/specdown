mod state;

use crate::results::test_result::TestResult;
use crate::runner::state::State;
use crate::types::Action;

mod error;
mod executor;
mod file;
mod script;
mod verify;

use executor::{Executor, Shell};

pub use error::Error;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub enum RunEvent {
    SpecFileStarted(PathBuf),
    TestCompleted(TestResult),
    SpecFileCompleted { success: bool },
    ErrorOccurred(Error),
}

pub fn run_actions(spec_file: &Path, actions: &[Action], shell_command: &str) -> Vec<RunEvent> {
    let mut events = vec![RunEvent::SpecFileStarted(spec_file.to_path_buf())];
    let mut state = State::new();
    let run_events: Result<Vec<RunEvent>, Error> =
        run_all_actions(actions, shell_command, &mut state)
            .or_else(|error| Ok(vec![RunEvent::ErrorOccurred(error)]));

    events.append(&mut run_events.unwrap());

    events.push(RunEvent::SpecFileCompleted {
        success: state.is_success(),
    });

    events
}

fn run_all_actions(
    actions: &[Action],
    shell_command: &str,
    mut state: &mut State,
) -> Result<Vec<RunEvent>, Error> {
    let executor = Shell::new(shell_command)?;
    actions
        .iter()
        .map(|action| run_single_action(&mut state, &executor, action))
        .collect()
}

fn run_single_action(
    state: &mut State,
    executor: &Shell,
    action: &Action,
) -> Result<RunEvent, Error> {
    run_action(action, &state, executor).map(|result| {
        state.add_result(&result);
        RunEvent::TestCompleted(result)
    })
}

fn run_action(
    action: &Action,
    state: &State,
    executor: &dyn Executor,
) -> Result<TestResult, error::Error> {
    match action {
        Action::Script {
            script_name,
            script_code,
            expected_exit_code,
        } => script::run(script_name, script_code, expected_exit_code, executor),
        Action::Verify {
            source,
            expected_value,
        } => verify::run(source, expected_value, state),
        Action::CreateFile {
            file_path,
            file_content,
        } => Ok(file::run(file_path, file_content)),
    }
}
