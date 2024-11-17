use std::io;
use std::path::{Path, PathBuf};

use crate::parser::{EnvPolicy, FsPolicy, GlobalOptions, NetPolicy, RunData, UtsPolicy};
use anyhow::{Context, Result};
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
        uid_map: "0 1000 1".to_string(),
        gid_map: "0 1000 1".to_string(),
        isolate_mnt: true,
        isolate_uts: true,
        isolate_net: true,
        isolate_ipc: true,
        isolate_user: true,
        env: vec![],
        unset_env_vars: true,
    };

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
                    let e1 = io::Error::new(io::ErrorKind::InvalidData, "Could not get image path");
                    let e2 = io::Error::new(io::ErrorKind::InvalidData, "Could not get image path");
                    let imgpath = Path::new(&image)
                        .file_name()
                        .ok_or(e1)?
                        .to_str()
                        .ok_or(e2)?;
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
