use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser, Clone)]
#[clap(name = "korobok", version)]
pub struct KorobokOptions {
    #[clap(flatten)]
    pub global_opts: GlobalOptions,

    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Args, Clone)]
pub struct GlobalOptions {
    #[clap(long)]
    /// Specify where container rootfs is copied (if it is copied at all).
    ///
    /// The default is creating a temporary directory
    pub run_dir: Option<String>,
}

#[derive(Debug, Subcommand, Clone)]
pub enum Command {
    /// Create a container and run a command inside of it
    Run(RunData),
}

#[derive(Debug, Args, Clone)]
pub struct RunData {
    #[clap(long, default_value_t = EnvPolicy::Clear)]
    #[arg(value_enum)]
    /// Control how envvars are handled when creating a container
    pub env: EnvPolicy,
    #[clap(long, default_value_t = NetPolicy::Sandbox)]
    #[arg(value_enum)]
    /// Control the new container's network access
    pub net: NetPolicy,
    #[clap(long, default_value_t = UtsPolicy::Sandbox)]
    #[arg(value_enum)]
    /// Control whether the container can acces AND ALTER hostname settings of the host machine
    pub uts: UtsPolicy,
    #[clap(long, default_value_t = FsPolicy::Copy)]
    #[arg(value_enum)]
    pub fs: FsPolicy,
    #[clap(long)]
    /// Raw uid_map that is passed into container (overrides --usr)
    pub uid_map: Option<String>,
    #[clap(long)]
    /// Raw gid_map that is passed into container (overrides --usr)
    pub gid_map: Option<String>,
    #[clap(long, default_value_t = UsrPolicy::Root)]
    #[arg(value_enum)]
    /// Control how users are mapped from host to container
    pub usr: UsrPolicy,
    /// Path to a directory containing container rootfs
    pub image: Option<String>,
    /// Command to run in container (better to pass after --)
    pub cmd: Vec<String>,
    #[clap(short = 'e')]
    /// Pass is a VARNAME:VALUE pair after it to set environment variables. Can be used multiple
    /// times:
    ///
    /// korobok run -e "KEY1:VALUE1" -e "KEY2:VALUE2" container_fs sh
    pub environment: Vec<String>,
    #[clap(long)]
    /// Do not detach stdout, stdin and stderr (this will not print the container id)
    pub no_detach: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum EnvPolicy {
    /// Preserve env values from host machine
    Preserve,
    /// Remove all env values before running entry command
    Clear,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum NetPolicy {
    /// Allow containers to use host interfaces
    Host,
    /// Hide all interfaces from container
    Sandbox,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum UtsPolicy {
    /// Allow containers to use AND ALTER host machine settings
    Host,
    /// Create separate uts namespace for containers
    Sandbox,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum FsPolicy {
    /// Copy the container rootfs to a separate location and run everything there
    Copy,
    /// Run the container in the provided directory
    NoCopy,
    /// Run the container in the host filesystem
    Host,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum UsrPolicy {
    /// Map effective UID&GID to root in container
    Root,
}
