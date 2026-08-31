//! Child processes, started the same way everywhere.

use std::ffi::OsStr;
use std::process::Command;

/// A command that starts without a console window of its own.
///
/// Windows gives every process it starts a console unless told otherwise, and this
/// application starts a lot of them: FFmpeg for probing and normalising, whisper
/// for a transcript, the llama server, a PowerShell line asking how much memory
/// the machine has. Each one flashed a black window in front of whatever somebody
/// was reading. `CREATE_NO_WINDOW` is what suppresses it.
///
/// The flag is written out rather than taken from a crate, because a dependency
/// for one constant is a dependency to keep updated for one constant. It has no
/// meaning off Windows, where this is a plain `Command` and the argument is unused.
pub(crate) fn command(program: impl AsRef<OsStr>) -> Command {
    #[allow(unused_mut)]
    let mut command = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000);
    }
    command
}
