use libkorobok::options::RunOptions;
use libkorobok::run_container::run_container;

fn main() {
    let opts = RunOptions {
        uid_map: "0 1000 1".to_string(),
        gid_map: "0 1000 1".to_string(),
        container_mount_point: "container_fs".to_string(),
        isolate_mnt: true,
        isolate_uts: true,
        isolate_user: true,
        unset_env_vars: true,
        env: vec![
            ("VAR1".to_string(), "VALUE1".to_string()),
            ("VAR2".to_string(), "VALUE2".to_string()),
        ],
    };

    run_container(&opts, &["/bin/sh".to_owned()]).unwrap();
}
