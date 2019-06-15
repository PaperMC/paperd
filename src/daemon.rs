extern crate nix;

use nix::unistd::{close, fork, setsid, ForkResult};
use std::os::unix::io::AsRawFd;

pub enum Status {
    CONTINUE,
    QUIT,
}

/// Forks the currently running process and returns either an error int (to exit with) or a `Status`
/// telling the caller whether to exit or to continue. The parent process should quit, with the
/// child process continuing, now as a separate daemon process.
pub fn run_daemon() -> Result<Status, i32> {
    // Create a new pid and execute from there
    match fork() {
        Ok(ForkResult::Parent { child, .. }) => {
            // Continue in the child, we're done in the parent
            return Ok(Status::QUIT);
        }
        Ok(ForkResult::Child) => {}
        Err(_) => {
            eprintln!("Fork failed");
            return Err(1);
        }
    }

    // At this point we are the forked process

    // create a new session, making this the leader of a new process group, preventing this from
    // becoming an orphan
    match setsid() {
        Ok(_) => {}
        Err(_) => {
            eprintln!("Failed to start a new session");
            return Err(1);
        }
    };

    // Close stdin, stdout, stderr; we won't be using them from here on
    close_fd(std::io::stdin());
    close_fd(std::io::stdout());
    close_fd(std::io::stderr());

    return Ok(Status::CONTINUE);
}

fn close_fd<T: AsRawFd>(fd: T) {
    match close(fd.as_raw_fd()) {
        _ => {}
    }
}
