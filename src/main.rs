use clone3::Clone3;
use interprocess::unnamed_pipe::{self, pipe};
use libc::{setgid, setuid, syscall, SYS_pivot_root};
use std::env::set_current_dir;
use std::ffi::CString;
use std::fs::DirBuilder;
use std::fs::File;
use std::io::ErrorKind;
use std::io::{prelude::*, BufReader};
use std::os::unix::fs::DirBuilderExt;
use std::process::exit;
use std::process::{Command, Stdio};
use std::{env, os::fd::OwnedFd};
use sys_mount::unmount;
use sys_mount::{Mount, MountFlags, UnmountFlags};

fn container_setup(pid: i32, tx: OwnedFd, rx: OwnedFd) {
    let mut tx = unnamed_pipe::Sender::from(tx);
    let mut rx = BufReader::new(unnamed_pipe::Recver::from(rx));
    eprintln!("[setup] Initialized communication and started container process");
    prepare_user_ns(pid).unwrap();
    eprintln!("[setup] Set up user namespace");
    eprintln!("[setup] Ready");
    tx.write_all(b"ready\n").unwrap();
    let mut s = String::new();
    loop {
        rx.read_line(&mut s).unwrap();
        match s.trim() {
            "finish" => {
                eprintln!("[setup] Finishing");
                return;
            }
            m => eprintln!("[setup] Got invalid message: `{:?}`", m),
        }
    }
}

fn container_main(cmd: &[String], tx: OwnedFd, rx: OwnedFd) {
    let mut tx = unnamed_pipe::Sender::from(tx);
    let mut rx = BufReader::new(unnamed_pipe::Recver::from(rx));
    eprintln!("[container] Initialized commmunication, waiting for setup to finish");
    let mut s = String::new();
    loop {
        rx.read_line(&mut s).unwrap();
        if s.trim() == "ready" {
            eprintln!("[container] Got ready message, executing entry command");
            break;
        }
    }
    eprintln!("[container] Prepare mount namespace");
    unsafe { prepare_mnt_ns() }
    // ensure we are root in the container
    unsafe {
        setuid(0);
        setgid(0);
    }
    Command::new(&cmd[0])
        .args(&cmd[1..])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .unwrap();
    eprintln!("[container] Finishing");
    tx.write_all(b"finish\n").unwrap();
}

/*
fn main() {
    let cmd = &env::args().collect::<Vec<_>>()[1..];
    if cmd.len() == 0 {
        eprintln!("You should specify the command: korobok `cmd` [args]");
        return;
    }

    let mut pidfd = -1;
    let mut clone3 = Clone3::default();
    clone3
        .flag_pidfd(&mut pidfd)
        .flag_newuts()
        .flag_newuser()
        .flag_newns()
        .flag_newpid();

    let (p_tx, c_rx) = pipe().unwrap();
    let (c_tx, p_rx) = pipe().unwrap();

    match unsafe { clone3.call() }.unwrap() {
        0 => container_main(cmd, c_tx.into(), c_rx.into()),
        pid => container_setup(pid, p_tx.into(), p_rx.into()),
    }
}
*/
