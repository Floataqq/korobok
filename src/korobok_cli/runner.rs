use crate::parser::{EnvPolicy, FsPolicy, GlobalOptions, NetPolicy, RunData, UsrPolicy, UtsPolicy};
use anyhow::{Context, Result};
use libc::{getegid, geteuid};
use libkorobok::container_dir::ContainerDir;
use libkorobok::options::RunOptions as Options;
use libkorobok::run_container;
use std::path::Path;
use std::process::Command;

fn prepare_container_rootfs(rd: &Path, image: &str) -> Result<()> {
    Command::new("cp")
        .arg("-r")
        .arg(image)
        .arg(rd)
        .output()
        .with_context(|| "Could not run cp to copy container rootfs where needed")?;
    Ok(())
}

fn run_in_dir(
    root_dir: &str,
    cmd: &[String],
    opts: &mut Options,
    image: Option<String>,
) -> Result<String> {
    let image = image.clone().with_context(|| {
        "You can't use --fs=run-copy (default value) without passing the image argument!"
    })?;
    let mut d = ContainerDir::new(root_dir)
        .with_context(|| "Could not create container runtime directory")?;
    prepare_container_rootfs(d.as_ref(), &image)
        .with_context(|| "Could not prepare container rootfs")?;
    let imgpath = Path::new(&image)
        .file_name()
        .with_context(|| "Could not get image path")?
        .to_str()
        .with_context(|| "Could not convert image path to string")?;
    opts.container_mount_point = d
        .as_ref()
        .join(imgpath)
        .to_str()
        .with_context(|| "Could not crate container mount point path")?
        .to_owned();
    run_container(&opts, cmd).with_context(|| "Could not run the container")?;
    let _ = d.close();
    Ok(d.id.clone())
}

pub fn run(run_data: RunData, global_opts: GlobalOptions) -> Result<String> {
    let mut opts = Options {
        container_mount_point: "".to_string(),
        uid_map: "".to_string(),
        gid_map: "".to_string(),
        isolate_mnt: true,
        isolate_uts: true,
        isolate_net: true,
        isolate_ipc: true,
        isolate_user: true,
        env: run_data
            .environment
            .into_iter()
            .map(|e| {
                let kv: Vec<&str> = e.split(":").collect();
                assert!(kv.len() == 2, "Invalid env arg: {e}");
                (kv[0].to_owned(), kv[1].to_owned())
            })
            .collect(),
        unset_env_vars: true,
        detach: !run_data.no_detach,
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

    let cmd = run_data.cmd.as_slice();
    let id = match run_data.fs {
        FsPolicy::Host => {
            opts.isolate_mnt = false;
            run_container(&opts, cmd).with_context(|| "Could not run the container")?;
            None
        }
        FsPolicy::NoCopy => {
            opts.container_mount_point = run_data
                .image
                .with_context(|| "You can't use --fs=run without passing the image argument!")?;
            run_container(&opts, cmd).with_context(|| "Could not run the container")?;
            None
        }
        FsPolicy::Copy => match global_opts.run_dir {
            None => Some(run_in_dir(
                "/tmp/korobok/",
                &cmd,
                &mut opts,
                run_data.image,
            )?),
            Some(root_dir) => Some(run_in_dir(&root_dir, &cmd, &mut opts, run_data.image)?),
        },
    };

    if !opts.detach {
        Ok(id.unwrap_or("".to_owned()))
    } else {
        Ok("".to_owned())
    }
}
