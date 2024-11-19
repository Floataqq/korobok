use std::io;
use std::path::{Path, PathBuf};

use crate::parser::{EnvPolicy, FsPolicy, GlobalOptions, NetPolicy, RunData, UsrPolicy, UtsPolicy};
use anyhow::{anyhow, Context, Result};
use libc::{getegid, geteuid};
use libkorobok::options::RunOptions as Options;
use libkorobok::run_container;
use std::process::Command;
use tempdir::TempDir;

fn prepare_container_rootfs(rd: &Path, image: &str) -> Result<()> {
    Command::new("cp")
        .arg("-r")
        .arg(image)
        .arg(rd)
        .output()
        .with_context(|| "Could not run cp to copy container rootfs where needed")?;
    Ok(())
}

pub fn run(run_data: RunData, global_opts: GlobalOptions) -> Result<()> {
    // ugly scope hack :(
    let rd = TempDir::new("korobok_container")
        .with_context(|| "Could not create tempdir to deplay container in")?;

    let mut opts = Options {
        container_mount_point: "".to_string(),
        uid_map: "".to_string(),
        gid_map: "".to_string(),
        isolate_mnt: true,
        isolate_uts: true,
        isolate_net: true,
        isolate_ipc: true,
        isolate_user: true,
        env: vec![],
        unset_env_vars: true,
    };

    match run_data.usr {
        UsrPolicy::Root => {
            let euid = unsafe { geteuid() };
            let egid = unsafe { getegid() };
            opts.uid_map = format!("0 {euid} 1");
            opts.gid_map = format!("0 {egid} 1");
        }
    }

    if let Some(uid_map) = run_data.uid_map {
        opts.uid_map = uid_map;
    }
    if let Some(gid_map) = run_data.gid_map {
        opts.gid_map = gid_map;
    }

    if run_data.env == EnvPolicy::Preserve {
        opts.unset_env_vars = false;
    }
    if run_data.uts == UtsPolicy::Host {
        opts.isolate_uts = false;
    }
    if run_data.net == NetPolicy::Host {
        opts.isolate_net = false;
    }
    match run_data.fs {
        FsPolicy::Host => {
            opts.isolate_mnt = false;
        }
        FsPolicy::Run => {
            opts.container_mount_point = run_data
                .image
                .with_context(|| "You can't use --fs=run without passing the image argument!")?;
        }
        FsPolicy::RunCopy => {
            match global_opts.run_dir {
                None => {
                    let image = run_data.image.with_context(|| {
                        "You can't use --fs=run-copy (default value) without passing the image argument!"
                    })?;
                    let imgpath = Path::new(&image)
                        .file_name()
                        .ok_or(anyhow!("Could not get image path"))?
                        .to_str()
                        .ok_or(anyhow!("Could not get image path"))?;
                    opts.container_mount_point = rd
                        .path()
                        .join(imgpath)
                        .to_str()
                        .with_context(|| {
                            "Could not turn container rootfs path to str when mounting in tempdir"
                        })?
                        .to_owned();
                    prepare_container_rootfs(rd.path(), &image)
                        .with_context(|| "Could not prepare container rootfs")?;
                }
                Some(_rd) => {
                    panic!("You can't yet set runtime directories, but it will be available in the future");
                    /*
                    let rd = Path::new(&rd);
                    let image = run_data.image.with_context(|| {
                        "You can't use --fs=run-copy (default value) without passing the image argument!"
                    })?;
                    prepare_container_rootfs(rd, &image)
                        .with_context(|| "Could not prepare container rootfs")?;
                    */
                }
            }
        }
    }
    let cmd = run_data.cmd.as_slice();

    run_container(&opts, cmd)?;

    Ok(())
}
