pub struct RunOptions {
    pub uid_map: String,
    pub gid_map: String,
    pub container_mount_point: String,
    pub isolate_mnt: bool,
    pub isolate_uts: bool,
    pub isolate_user: bool,
    pub isolate_net: bool,
    pub isolate_ipc: bool,
    // pub isolate_pid: bool,
    pub unset_env_vars: bool,
    pub env: Vec<(String, String)>,
    pub detach: bool,
}
