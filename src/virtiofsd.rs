use std::path::Path;
use std::path::PathBuf;
use std::process::Child;
use std::process::Command;
use std::process::Stdio;
use std::sync::LazyLock;
use std::time::Duration;
use std::time::Instant;

use anyhow::bail;
use anyhow::Context as _;
use anyhow::Result;
use log::error;

use crate::util::gen_sock;

const VIRTIOFSD_NAME: &str = "virtiofsd";
static VIRTIOFSD_PATH: LazyLock<Option<PathBuf>> = LazyLock::new(find_virtiofsd_exe);

fn find_virtiofsd_exe() -> Option<PathBuf> {
    // The virtiofsd binary can be found at `/usr/libexec/virtiofsd`
    // on Debian and Fedora, `/usr/lib/virtiofsd` on Arch.
    let dirs = ["/usr/libexec/", "/usr/lib/"];
    for dir in dirs {
        let path = Path::new(dir).join(VIRTIOFSD_NAME);
        if path.exists() {
            return Some(path);
        }
    }

    None
}

enum State {
    Command(Command),
    Process(Child),
}

pub(crate) struct Virtiofsd {
    /// Our main state.
    state: State,
    /// The path to the Unix domain socket used for communication.
    socket_path: PathBuf,
}

impl Virtiofsd {
    /// Create a `Virtiofsd` instance for sharing the given directory.
    pub fn new(shared_dir: &Path) -> Result<Self> {
        let virtiofsd_path = VIRTIOFSD_PATH
            .as_ref()
            .with_context(|| format!("`{VIRTIOFSD_NAME}` binary not found"))?;
        let socket_path = gen_sock(VIRTIOFSD_NAME);

        let mut command = Command::new(virtiofsd_path);
        let _ = command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .arg("--socket-path")
            .arg(&socket_path)
            .arg("--shared-dir")
            .arg(shared_dir)
            .arg("-ocache=always")
            .arg("--sandbox=none")
            .arg("--announce-submounts")
            .arg("--log-level=error");

        let slf = Self {
            state: State::Command(command),
            socket_path,
        };
        Ok(slf)
    }

    pub fn launch(&mut self) -> Result<()> {
        match &mut self.state {
            State::Command(command) => {
                let process = command
                    .spawn()
                    .with_context(|| format!("failed to spawn `{VIRTIOFSD_NAME}` instance"))?;

                self.state = State::Process(process)
            }
            State::Process(..) => {
                // Already launched. No-op.
            }
        }
        Ok(())
    }

    pub fn await_launched(&mut self) -> Result<()> {
        if let State::Command(..) = self.state {
            let () = self.launch()?;
        }

        match self.state {
            State::Command(..) => unreachable!(),
            State::Process(..) => {
                let now = Instant::now();
                let timeout = Duration::from_secs(5);

                while now.elapsed() < timeout {
                    if self.socket_path.exists() {
                        return Ok(());
                    }
                }
            }
        };

        bail!(
            "{VIRTIOFSD_NAME} socket `{}` did not appear in time",
            self.socket_path.display()
        )
    }

    #[inline]
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }
}

impl Drop for Virtiofsd {
    fn drop(&mut self) {
        if let State::Process(process) = &mut self.state {
            if let Err(err) = process.kill() {
                error!("failed to kill `{VIRTIOFSD_NAME}` instance: {err}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Check that we can discover the path to `virtiofsd`.
    #[test]
    fn virtiofsd_discovery() {
        let _path = VIRTIOFSD_PATH.as_ref().expect("failed to find virtiofsd");
    }
}
