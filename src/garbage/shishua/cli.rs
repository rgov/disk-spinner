use std::{
    io,
    process::{Child, Command, Stdio},
};

use anyhow::Context as _;

use crate::garbage::GarbageGenerator;

pub struct ShishuaCliGenerator {
    progress: Box<dyn Fn(u64)>,
    #[allow(dead_code)]
    child: Child,
    stdout: std::process::ChildStdout,
}
impl GarbageGenerator for ShishuaCliGenerator {}

impl ShishuaCliGenerator {
    pub fn new(seed: u64, progress: Box<dyn Fn(u64)>) -> anyhow::Result<Self> {
        let mut child = Command::new("shishua")
            .arg("--seed")
            .arg(format!("{seed:X}"))
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .spawn()
            .context("spawning shishua CLI tool")?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("Child process somehow has no stdout"))?;
        Ok(ShishuaCliGenerator {
            child,
            stdout,
            progress,
        })
    }
}

impl io::Read for ShishuaCliGenerator {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let result = self.stdout.read(buf);
        if let Ok(n) = result {
            (self.progress)(n as u64);
        }
        result
    }
}
