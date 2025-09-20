use std::thread::{self, JoinHandle};

use shi::{command::Command, error::ShiError, shell::Shell};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CreateShellError {
    #[error("Failed to register command {0:?}: {1:?}")]
    RegisterError(usize, ShiError),

    #[error("Got ShiError: {0:?}")]
    ShiError(#[from] ShiError),
}

pub fn create_shell<'a, S: 'a>(
    shell_state: S,
    commands: Vec<Command<'a, S>>,
) -> Result<Shell<'a, S>, CreateShellError> {
    let mut mut_shell = Shell::new_with_state("| ", shell_state)?;

    for (i, command) in commands.into_iter().enumerate() {
        mut_shell
            .register(command)
            .map_err(|e| CreateShellError::RegisterError(i, e))?;
    }

    Ok(mut_shell)
}

#[derive(Error, Debug)]
pub enum SpawnShellLoopThreadError {
    #[error("Failed to create shell: {0:?}")]
    CreateShellError(CreateShellError),

    #[error("Failed to update shell with error: {0:?}")]
    ShellUpdateError(ShiError),
}

pub fn spawn_shell_loop_thread<'a, S>(
    shell_state_fn: impl FnOnce() -> S + Send + 'static,
    commands_fn: impl FnOnce() -> Vec<Command<'a, S>> + Send + 'static,
    on_update_fn: impl Fn(usize) -> Option<()> + Send + 'static,
) -> JoinHandle<Result<(), SpawnShellLoopThreadError>> {
    thread::spawn(move || {
        let shell_state = shell_state_fn();
        let commands = commands_fn();

        let mut mut_shell = create_shell(shell_state, commands)
            .map_err(SpawnShellLoopThreadError::CreateShellError)?;

        let mut mut_i = 0;

        loop {
            match mut_shell.update() {
                Ok(do_update_shell) => {
                    if !do_update_shell {
                        break;
                    }

                    if on_update_fn(mut_i).is_none() {
                        break;
                    }

                    mut_i += 1;
                }
                Err(e) => return Err(SpawnShellLoopThreadError::ShellUpdateError(e)),
            }
        }

        mut_shell.finish().unwrap();

        log::info!("Bye!");

        Ok(())
    })
}
