// This file is part of the OOF project, released under the Creative Commons CC0
// https://creativecommons.org/publicdomain/zero/1.0/

use std::collections::HashMap;
use std::path::PathBuf;

use console::style;
use over::obj::Obj;

const COULD_NOT_BE_PARSED_AS_STRING: &'static str = "could not be parsed as a String";
const COULD_NOT_BE_PARSED_AS_OBJ: &'static str = "could not be parsed as an Object";
const MAINTAINER_OR_HOMEPAGE_REQUIRED: &'static str =
    "meta.maintainer and/or meta.homepage is required";

// TODO everything in this chunk needs to find a reusable home

#[derive(Debug, PartialEq, Eq)]
pub enum SchemaParsingError {
    Generic(&'static str),
    MissingOofInstruction,
    MalformedOofInstruction {
        field_name: String,
        problem: &'static str,
    },
    UnsupportedSchemaType(String),
    UnsupportedSchemaVersion {
        schema_type: String,
        requested_version: String,
    },
}

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

#[derive(Debug)]
pub struct OofFile {
    schema: OofFileSchema,
    meta: OofFileMeta,
}

#[derive(Debug)]
pub enum OofFileSchema {
    System20210801,
}

#[derive(Debug)]
pub struct OofFileMeta {
    maintainer: Option<OofFileMetaMaintainer>,
    homepage: Option<String>,
    license: OofFileLicense,
}

#[derive(Debug)]
pub struct OofFileMetaMaintainer {
    name: String,
    contact: Option<String>,
}

#[derive(Debug)]
pub enum OofFileLicense {
    Restricted,
    SPDXIdentifier(String),
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

pub fn from_over_obj(obj: &Obj) -> Result<OofFile, SchemaParsingError> {
    let oof_instruction_obj = match obj.get_obj(&"oof") {
        Ok(oof) => oof,
        Err(_) => {
            return Err(SchemaParsingError::MissingOofInstruction);
        }
    };

    let oof_schema = match parse_oof_schema_type(&oof_instruction_obj) {
        Ok(schema) => schema,
        Err(err) => {
            return Err(err);
        }
    };

    eprintln!(
        "{} parsed schema header: {:?}",
        style("successfully").green(),
        oof_schema
    );

    let oof_meta = match parse_oof_meta(&oof_instruction_obj) {
        Ok(meta) => meta,
        Err(err) => {
            return Err(err);
        }
    };

    eprintln!(
        "{} parsed meta header: {:?}",
        style("successfully").green(),
        oof_meta
    );

    Err(SchemaParsingError::Generic("rest not implemented"))
}

fn parse_oof_schema_type(oof: &Obj) -> Result<OofFileSchema, SchemaParsingError> {
    match oof.get_obj(&"schema") {
        Ok(schema) => match schema.get_str(&"type") {
            Ok(schema_type) => match schema_type.as_str() {
                "system" => match schema.get_str(&"version") {
                    Ok(version) => match version.as_str() {
                        "2021.08.01" => Ok(OofFileSchema::System20210801),
                        _ => Err(SchemaParsingError::UnsupportedSchemaVersion {
                            schema_type: schema_type,
                            requested_version: version,
                        }),
                    },
                    Err(_) => Err(SchemaParsingError::MalformedOofInstruction {
                        field_name: "schema.version".to_string(),
                        problem: COULD_NOT_BE_PARSED_AS_STRING,
                    }),
                },
                _ => Err(SchemaParsingError::UnsupportedSchemaType(schema_type)),
            },
            Err(_) => Err(SchemaParsingError::MalformedOofInstruction {
                field_name: "schema.type".to_string(),
                problem: COULD_NOT_BE_PARSED_AS_STRING,
            }),
        },
        Err(_) => Err(SchemaParsingError::MalformedOofInstruction {
            field_name: "schema".to_string(),
            problem: COULD_NOT_BE_PARSED_AS_OBJ,
        }),
    }
}

fn parse_oof_meta(oof: &Obj) -> Result<OofFileMeta, SchemaParsingError> {
    match oof.get_obj(&"meta") {
        Ok(meta) => {
            let maintainer = parse_oof_meta_maintainer(&meta);
            let homepage = meta.get_str(&"homepage");

            if !maintainer.is_ok() && !homepage.is_ok() {
                return Err(SchemaParsingError::MalformedOofInstruction {
                    field_name: "meta.maintainer".to_string(),
                    problem: MAINTAINER_OR_HOMEPAGE_REQUIRED,
                });
            }

            if let Ok(license) = meta.get_str(&"license") {
                let lowercased_license = license.to_lowercase();

                return Ok(OofFileMeta {
                    maintainer: maintainer.ok(),
                    homepage: homepage.ok(),
                    license: match lowercased_license.as_str() {
                        "restricted" | "proprietary" => OofFileLicense::Restricted,
                        _ => OofFileLicense::SPDXIdentifier(license),
                    },
                });
            } else {
                return Err(SchemaParsingError::MalformedOofInstruction {
                    field_name: "meta.license".to_string(),
                    problem: COULD_NOT_BE_PARSED_AS_STRING,
                });
            }
        }
        Err(_) => Err(SchemaParsingError::MalformedOofInstruction {
            field_name: "meta".to_string(),
            problem: COULD_NOT_BE_PARSED_AS_STRING,
        }),
    }
}

fn parse_oof_meta_maintainer(meta: &Obj) -> Result<OofFileMetaMaintainer, SchemaParsingError> {
    match meta.get_obj(&"maintainer") {
        Ok(maintainer) => match maintainer.get_str(&"name") {
            Ok(maintainer_name) => Ok(OofFileMetaMaintainer {
                name: maintainer_name,
                contact: match maintainer.get_str(&"contact") {
                    Ok(contact) => Some(contact),
                    Err(_) => None,
                },
            }),
            Err(_) => Err(SchemaParsingError::MalformedOofInstruction {
                field_name: "meta.maintainer.name".to_string(),
                problem: COULD_NOT_BE_PARSED_AS_STRING,
            }),
        },
        Err(_) => Err(SchemaParsingError::MalformedOofInstruction {
            field_name: "meta.maintainer".to_string(),
            problem: COULD_NOT_BE_PARSED_AS_OBJ,
        }),
    }
}
