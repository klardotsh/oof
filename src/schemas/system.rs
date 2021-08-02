// This file is part of the OOF project, released under the Creative Commons CC0
// https://creativecommons.org/publicdomain/zero/1.0/

use std::collections::HashMap;
use std::path::PathBuf;

use console::style;
use over::obj::Obj;

const COULD_NOT_BE_PARSED_AS_STRING: &'static str = "could not be parsed as a String";
const COULD_NOT_BE_PARSED_AS_OBJ: &'static str = "could not be parsed as an Object";
const COULD_NOT_BE_PARSED_AS_ARR: &'static str = "could not be parsed as a homogenous Array";
const COULD_NOT_DETERMINE_REPO_TYPE: &'static str = "could not determine repo type (eg. git)";
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
    ExtendingNonExistantRepo(String),
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
pub enum Executable {
    Discoverable(&'static str),
    UserProvided(String),
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
    using: UsingMap,
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

type UsingMap = HashMap<String, Using>;

#[derive(Debug)]
pub enum Using {
    Git {
        upstream: String,
        rev: Option<String>,
        shallow: bool,
        bin: Executable,
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

    let oof_schema = parse_oof_schema_type(&oof_instruction_obj)?;
    eprintln!(
        "{} parsed schema header: {:?}",
        style("successfully").green(),
        oof_schema
    );

    let oof_meta = parse_oof_meta(&oof_instruction_obj)?;
    eprintln!(
        "{} parsed meta header: {:?}",
        style("successfully").green(),
        oof_meta
    );

    // TODO eventually, actually parse what the user asked for here. For now, since there is
    // exactly one supported value, let's just hard-code and move on.
    let target = Target {
        target_type: TargetType::TargetSelf,
    };
    eprintln!(
        "{} hard-coded target block (lol): {:?}",
        style("successfully").green(),
        target
    );

    let using = parse_using(&obj)?;
    eprintln!(
        "{} parsed using: {:?}",
        style("successfully").green(),
        using
    );

    let extends = parse_extends(&obj, &using)?;
    eprintln!(
        "{} parsed extends: {:?}",
        style("successfully").green(),
        extends
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
                        field_name: "oof.schema.version".to_string(),
                        problem: COULD_NOT_BE_PARSED_AS_STRING,
                    }),
                },
                _ => Err(SchemaParsingError::UnsupportedSchemaType(schema_type)),
            },
            Err(_) => Err(SchemaParsingError::MalformedOofInstruction {
                field_name: "oof.schema.type".to_string(),
                problem: COULD_NOT_BE_PARSED_AS_STRING,
            }),
        },
        Err(_) => Err(SchemaParsingError::MalformedOofInstruction {
            field_name: "oof.schema".to_string(),
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
                    field_name: "oof.meta.maintainer".to_string(),
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
                    field_name: "oof.meta.license".to_string(),
                    problem: COULD_NOT_BE_PARSED_AS_STRING,
                });
            }
        }
        Err(_) => Err(SchemaParsingError::MalformedOofInstruction {
            field_name: "oof.meta".to_string(),
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

fn parse_using(config: &Obj) -> Result<UsingMap, SchemaParsingError> {
    match config.get_obj(&"using") {
        Ok(using) => {
            let mut result: UsingMap = HashMap::with_capacity(using.len());

            for (repo, config) in using.iter() {
                if let Ok(config_obj) = config.get_obj() {
                    if let Ok(git) = config_obj.get_str(&"git") {
                        result.insert(
                            repo.clone(),
                            Using::Git {
                                upstream: git,
                                rev: config_obj.get_str(&"rev").ok(),
                                shallow: config_obj.get_bool(&"shallow").unwrap_or(false),
                                bin: match config_obj.get_str(&"bin") {
                                    Ok(bin) => Executable::UserProvided(bin),
                                    Err(_) => Executable::Discoverable("git"),
                                },
                            },
                        );
                        continue;
                    }
                }

                return Err(SchemaParsingError::MalformedOofInstruction {
                    field_name: format!("using.{}", repo),
                    problem: COULD_NOT_DETERMINE_REPO_TYPE,
                });
            }

            Ok(result)
        }
        Err(_) => Err(SchemaParsingError::MalformedOofInstruction {
            field_name: "using".to_string(),
            problem: COULD_NOT_BE_PARSED_AS_OBJ,
        }),
    }
}

fn parse_extends<'using>(
    config: &Obj,
    using: &'using UsingMap,
) -> Result<Vec<Extends<'using>>, SchemaParsingError> {
    match config.get_arr(&"extends") {
        Ok(extends) => {
            let mut result = Vec::with_capacity(extends.len());

            for (idx, eraw) in extends.iter().enumerate() {
                let eobj = match eraw.get_obj() {
                    Ok(obj) => obj,
                    Err(_) => {
                        return Err(SchemaParsingError::MalformedOofInstruction {
                            field_name: format!("extends[{}]", idx),
                            problem: COULD_NOT_BE_PARSED_AS_OBJ,
                        });
                    }
                };

                let repo = match eobj.get_str(&"repo") {
                    Ok(val) => val,
                    Err(_) => {
                        return Err(SchemaParsingError::MalformedOofInstruction {
                            field_name: format!("extends[{}].repo", idx),
                            problem: COULD_NOT_BE_PARSED_AS_STRING,
                        });
                    }
                };

                if !using.contains_key(&repo) {
                    return Err(SchemaParsingError::ExtendingNonExistantRepo(repo.clone()));
                }

                let path = match eobj.get_str(&"path") {
                    Ok(path) => path,
                    Err(_) => {
                        return Err(SchemaParsingError::MalformedOofInstruction {
                            field_name: format!("extends[{}].path", idx),
                            problem: COULD_NOT_BE_PARSED_AS_STRING,
                        });
                    }
                };

                result.push(Extends {
                    repo: using.get(&repo).unwrap(),
                    path: path.clone(),
                    pick: eobj
                        .get_arr(&"pick")
                        .and_then(|picks| Ok(picks.vec_ref().to_vec()))
                        .and_then(|objs| objs.iter().map(|obj| obj.get_str()).collect())
                        .ok(),
                    omit: eobj
                        .get_arr(&"omit")
                        .and_then(|omits| Ok(omits.vec_ref().to_vec()))
                        .and_then(|objs| objs.iter().map(|obj| obj.get_str()).collect())
                        .ok(),
                });
            }

            Ok(result)
        }
        Err(_) => Err(SchemaParsingError::MalformedOofInstruction {
            field_name: "extends".to_string(),
            problem: COULD_NOT_BE_PARSED_AS_ARR,
        }),
    }
}
