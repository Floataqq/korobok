use crate::namespaces;
use crate::options::RunOptions as Options;
use anyhow::{Context, Result};
use clone3::Clone3;
use interprocess::unnamed_pipe::{pipe, Recver, Sender};
use libc;
use std::{
    io::{self, BufRead, BufReader, Write},
    os::fd::OwnedFd,
    process::{Command, Stdio},
};

fn container_setup(opts: &Options, container_pid: i32, tx: OwnedFd, rx: OwnedFd) -> Result<()> {
    let mut tx = Sender::from(tx);
    let mut rx = BufReader::new(Recver::from(rx));
    if opts.isolate_user {
        namespaces::prepare_user_ns(opts, container_pid)
            .with_context(|| "[setup] Could not prepare user namespace")?;
    }
    tx.write_all(b"ready\n")
        .with_context(|| "[setup] Could not send ready message to container")?;
    let mut s = String::new();
    loop {
        rx.read_line(&mut s)
            .with_context(|| "[setup] Could not read message from container")?;
        match s.trim() {
            "finish" => return Ok(()),
            _ => continue,
        }
    }
}

unsafe fn container_main(opts: &Options, cmd: &[String], tx: OwnedFd, rx: OwnedFd) -> Result<()> {
    let mut tx = Sender::from(tx);
    let mut rx = BufReader::new(Recver::from(rx));
    let mut s = String::new();
    loop {
        rx.read_line(&mut s)
            .with_context(|| "[container] Coudl not read message from setup")?;
        if s.trim() == "ready" {
            break;
        }
    }

    if opts.isolate_mnt {
        namespaces::prepare_mnt_ns(opts)
            .with_context(|| "[container] Could not prepare mount namespace")?;
    }

    // ensure we are root in the container
    libc::setuid(0);
    libc::setgid(0);

    let stdio_mode = if opts.detach {
        (Stdio::piped(), Stdio::piped(), Stdio::piped())
    } else {
        (Stdio::inherit(), Stdio::inherit(), Stdio::inherit())
    };
    if opts.isolate_net {
        Command::new(&cmd[0])
            .args(&cmd[1..])
            .stdin(stdio_mode.0)
            .stdout(stdio_mode.1)
            .stderr(stdio_mode.2)
            .output()
            .with_context(|| "[container] Could not run entry command")?;
    }

    tx.write_all(b"finish\n")
        .with_context(|| "[container] Could not send finish message to setup")?;
    Ok(())
}

pub fn run_container(opts: &Options, cmd: &[String]) -> Result<()> {
    if cmd.len() == 0 {
        return Err(
            io::Error::new(io::ErrorKind::InvalidInput, "You should provide a command").into(),
        );
    }

    let mut clone3 = Clone3::default();
    if opts.isolate_uts {
        clone3.flag_newuts();
    }
    if opts.isolate_user {
        clone3.flag_newuser();
    }
    if opts.isolate_mnt {
        clone3.flag_newns();
        clone3.flag_newpid();
    }
    if opts.isolate_net {
        clone3.flag_newnet();
    }
    if opts.isolate_ipc {
        clone3.flag_newipc();
    }

    let (p_tx, c_rx) = pipe().with_context(|| "Could not create IPC pipe")?;
    let (c_tx, p_rx) = pipe().with_context(|| "Could not create IPC pipe")?;

    unsafe {
        match clone3
            .call()
            .with_context(|| "Could not call clone3 with these options")?
        {
            0 => container_main(opts, cmd, c_tx.into(), c_rx.into())
                .with_context(|| "Something broke in the container process")?,
            pid => container_setup(opts, pid, p_tx.into(), p_rx.into())
                .with_context(|| "Smoething broke in the setup process")?,
        }
    }
    Ok(())
}
