
use std::process::{Command, Stdio, ChildStdin};

use crate::errors::{AppError, AppResultU};



pub fn with_pager<F>(f: F) -> AppResultU
where F: FnOnce(&mut ChildStdin) -> AppResultU {
    let mut c = Command::new("less");
    c.args(&["--quit-if-one-screen", "--RAW-CONTROL-CHARS", "--no-init"]);
    c.stdin(Stdio::piped());
    c.stdout(Stdio::inherit());

    let mut child = c.spawn()?;
    let stdin: &mut ChildStdin = child.stdin.as_mut().ok_or(AppError::Unexpect("Failed to open stdin for pager"))?;
    f(stdin)?;
    child.wait()?;

    Ok(())
}
