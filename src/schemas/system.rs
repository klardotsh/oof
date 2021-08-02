// This file is part of the OOF project, released under the Creative Commons CC0
// https://creativecommons.org/publicdomain/zero/1.0/

use std::collections::HashMap;
use std::path::PathBuf;

// TODO everything in this chunk needs to find a reusable home

#[derive(Debug)]
pub enum ErrorBehavior {
    Error,
    Warn,
}

#[derive(Debug)]
pub enum IgnorableErrorBehavior {
    Error,
    Ignore,
    Warn,
}

#[derive(Debug)]
pub enum SecurableInput {
    Raw(String),
    FilePlaintext(String),
    FileGpgNear {
        path: String,
        key: String,
        executable: Option<String>,
    },
    PromptOnce,
    PromptAlways,
}

// below here should all be schema-specific

#[derive(Debug)]
pub struct SystemSchema20210801<'config> {
    target: Target,
    using: HashMap<String, Using>,
    extends: Option<Vec<Extends<'config>>>,
    disks: Option<Vec<Disk>>,
    linux_kernels: Option<Vec<LinuxKernel>>,
    users: Option<HashMap<&'config str, User<'config>>>,
    groups: Option<HashMap<&'config str, Group>>,
    shells: Option<HashMap<String, Shell>>,
    privesc: Option<Privesc>,
    intentpkgs: Option<Vec<IntentPkg>>,
    rawpkgs: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct Target {
    target_type: TargetType,
}

#[derive(Debug)]
pub enum TargetType {
    TargetSelf,
}

#[derive(Debug)]
pub enum Using {
    Git {
        rev: Option<String>,
        shallow: bool,
        bin: String,
    },
}

#[derive(Debug)]
pub struct Extends<'outer_config> {
    repo: &'outer_config Using,
    path: String,
    pick: Option<Vec<String>>,
    omit: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct Disk {
    source: String,
    mountpoint: String,
    disk_type: DiskType,
    options: Vec<String>,
    dump: bool,
    fsck_order: FsckOrder,
    install_userspace_utils: bool,
    install_kernel_modules: bool,
}

// this is unlikely to be exhaustive, but we can try to enumerate as many as we come across
#[derive(Debug)]
pub enum DiskType {
    Bcachefs,
    Btrfs,
    Ext2,
    Ext3,
    Ext4,
    Jfs,
    Nilfs2,
    Ntfs,
    Swap,
    Tmpfs,
    Vfat,
    Xfs,
    Zfs,
}

#[derive(Debug)]
pub enum FsckOrder {
    Disabled = 0,
    First = 1,
    Next = 2,
}

#[derive(Debug)]
pub struct LinuxKernel {
    series: LinuxKernelSeries,
    versions: semver::VersionReq,
    install_headers: bool,
    install_firmware: Option<bool>,
}

#[derive(Debug)]
pub enum LinuxKernelSeries {
    Default,
    Other(String),
}

#[derive(Debug)]
pub struct User<'outer_config> {
    name: String,
    is_system: bool,
    uid: Option<u32>,
    main_group: String,
    extra_groups: Option<Vec<String>>,
    full_name: Option<String>,
    shell: &'outer_config UserShellRef,
    install_missing_shell: bool,
    password: Option<SecurableInput>,
    state_stub: bool,
    match_by: UserOrGroupMatchBy,
    not_matched_error_behavior: IgnorableErrorBehavior,
    prune_on_removal: bool,
}

#[derive(Debug)]
pub enum UserShellRef {
    AbsolutePath(PathBuf),
    BinName(String),
}

#[derive(Debug)]
pub enum UserOrGroupMatchBy {
    Name,
    ID,
}

#[derive(Debug)]
pub struct Group {
    name: String,
    is_system: bool,
    gid: Option<u32>,
    state_stub: bool,
    match_by: UserOrGroupMatchBy,
    not_matched_error_behavior: IgnorableErrorBehavior,
    prune_on_removal: bool,
}

#[derive(Debug)]
pub struct Shell {
    install_completion: bool,
    install_completion_error_behavior: ErrorBehavior,
    system_config_file: Option<PathBuf>,
}

#[derive(Debug)]
pub struct Privesc {
    method: PrivEscMethod,
    config_file: Option<PathBuf>,
}

#[derive(Debug)]
pub enum PrivEscMethod {
    Doas,
    Sudo,
}

#[derive(Debug)]
pub struct IntentPkg {
    name: String,
    opts: Option<HashMap<String, IntentPkgOpt>>,
}

#[derive(Debug)]
pub enum IntentPkgOpt {
    Null,
    Boolean(bool),
    Number(isize),
    String(String),
    List(Box<IntentPkgOpt>),
    Map(HashMap<String, Box<IntentPkgOpt>>),
}
