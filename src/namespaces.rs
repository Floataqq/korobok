use crate::options::RunOptions as Options;
use crate::syscall;
use anyhow::{Context, Result};
use std::env::{self, remove_var, set_current_dir, set_var};
use std::fs::{DirBuilder, File};
use std::io::{ErrorKind, Write};
use std::os::unix::fs::DirBuilderExt;
use sys_mount::{unmount, Mount, MountFlags, UnmountFlags};

pub fn prepare_user_ns(opts: &Options, pid: i32) -> Result<()> {
    File::create(format!("/proc/{pid}/uid_map"))
        .with_context(|| "[setup] Could not open uid_map")?
        .write(opts.uid_map.as_bytes())
        .with_context(|| "[setup] Could not write to uid_map")?;

    File::create(format!("/proc/{pid}/setgroups"))
        .with_context(|| "[setup] Could not open setgroups")?
        .write(b"deny")
        .with_context(|| "[setup] Could not write to setgroups")?;

    File::create(format!("/proc/{pid}/gid_map"))
        .with_context(|| "[setup] Could not open gid_map")?
        .write(opts.gid_map.as_bytes())
        .with_context(|| "[setup] Could not write to gid_map")?;
    Ok(())
}

pub unsafe fn prepare_mnt_ns(opts: &Options) -> Result<()> {
    Mount::builder()
        .fstype("ext4")
        .flags(MountFlags::BIND)
        .mount(&opts.container_mount_point, &opts.container_mount_point)
        .with_context(|| "[container] Could not mount container root fs")?;

    set_current_dir(&opts.container_mount_point)
        .with_context(|| "[container] Could not change directory to mount point")?;

    let mut builder = DirBuilder::new();
    builder.mode(0o777);
    let res = builder.create("put_old");
    match res {
        Ok(()) => {}
        Err(e) => {
            if e.kind() != ErrorKind::AlreadyExists {
                return Err(e)
                    .with_context(|| format!("[container] Could not create `put_old`"))?;
            }
        }
    }

    syscall::pivot_root(".", "put_old").with_context(|| "[container] Could not pivot root")?;

    set_current_dir("/")
        .with_context(|| "[container] Could not change directory to container root")?;

    builder.mode(0o555);
    let res = builder.create("/proc");
    match res {
        Ok(()) => {}
        Err(e) => {
            if e.kind() != ErrorKind::AlreadyExists {
                return Err(e).with_context(|| "[container] Could not create /proc");
            }
        }
    }

    Mount::builder()
        .fstype("proc")
        .mount("proc", "/proc")
        .with_context(|| "[container] Could not mount /proc")?;

    unmount("put_old", UnmountFlags::DETACH)
        .with_context(|| "[container] Could not unmount `put_old`")?;

    if opts.unset_env_vars {
        for (name, _val) in env::vars() {
            remove_var(name);
        }
    }
    for (name, val) in &opts.env {
        set_var(name, val);
    }

    Ok(())
}
