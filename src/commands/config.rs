// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use inquire::Confirm;
use inquire::Password;
use inquire::PasswordDisplayMode;
use inquire::Select;
use inquire::Text;
use inquire::error::InquireError;
use toml::Table;
use toml::Value;

use crate::config::Config;
use crate::params::config::ConfigParams;

#[derive(Debug, clap::Parser)]
#[command(
    name = "config",
    about = "Manage oli configuration",
    disable_version_flag = true
)]
pub struct ConfigCmd {
    #[command(subcommand)]
    subcommand: ConfigSubcommand,
}

#[derive(Debug, clap::Subcommand)]
enum ConfigSubcommand {
    Add(ConfigAddCmd),
    View(ConfigViewCmd),
}

impl ConfigCmd {
    pub fn run(self) -> Result<()> {
        match self.subcommand {
            ConfigSubcommand::Add(cmd) => cmd.run(),
            ConfigSubcommand::View(cmd) => cmd.run(),
        }
    }
}

#[derive(Debug, clap::Args)]
#[command(name = "add", about = "Add a configuration profile interactively")]
pub struct ConfigAddCmd {
    #[command(flatten)]
    pub config_params: ConfigParams,
}

impl ConfigAddCmd {
    pub fn run(self) -> Result<()> {
        let selected_schema = match Select::new("Select a schema", schema_names()).prompt() {
            Ok(schema) => schema,
            Err(InquireError::OperationCanceled | InquireError::OperationInterrupted) => {
                return Ok(());
            }
            Err(err) => return Err(err.into()),
        };
        let schema = profile_schema(&selected_schema)
            .ok_or_else(|| anyhow!("unknown schema selected: {selected_schema}"))?;

        let profile_name = match prompt_required_text("Profile name")? {
            Some(profile_name) => profile_name,
            None => return Ok(()),
        };

        if config_profile_exists(&self.config_params.config, &profile_name)? {
            let overwrite = match Confirm::new(&format!(
                "Profile `{profile_name}` already exists. Overwrite it?"
            ))
            .with_default(false)
            .prompt()
            {
                Ok(overwrite) => overwrite,
                Err(InquireError::OperationCanceled | InquireError::OperationInterrupted) => {
                    return Ok(());
                }
                Err(err) => return Err(err.into()),
            };
            if !overwrite {
                return Ok(());
            }
        }

        let mut profile = HashMap::from([("type".to_string(), schema.name.to_string())]);
        for field in ordered_prompt_fields(schema) {
            let Some(value) = prompt_field(field)? else {
                return Ok(());
            };
            if let Some(value) = value {
                profile.insert(field.name.to_string(), value);
            }
        }

        write_profile(&self.config_params.config, &profile_name, &profile)?;
        println!(
            "Added profile `{profile_name}` to {}",
            self.config_params.config.display()
        );

        Ok(())
    }
}

#[derive(Debug, clap::Args)]
#[command(name = "view", about = "Inspect configured profiles")]
pub struct ConfigViewCmd {
    #[command(flatten)]
    pub config_params: ConfigParams,
}

impl ConfigViewCmd {
    pub fn run(self) -> Result<()> {
        let cfg = Config::load(&self.config_params.config)?;
        let mut profiles = cfg.profile_names();

        if profiles.is_empty() {
            println!(
                "No configuration profiles found in {}",
                self.config_params.config.display()
            );
            return Ok(());
        }

        profiles.sort();
        let selected = match Select::new("Select a profile to view", profiles).prompt() {
            Ok(profile) => profile,
            Err(InquireError::OperationCanceled | InquireError::OperationInterrupted) => {
                return Ok(());
            }
            Err(err) => return Err(err.into()),
        };

        let Some(options) = cfg.profile(&selected) else {
            // Profiles are loaded up front, so missing here would indicate a race.
            println!("Profile `{selected}` is no longer available.");
            return Ok(());
        };

        println!("Profile: {selected}");
        println!("--------------------------------");
        for (k, v) in ordered_entries(options) {
            println!("{k} = {v}");
        }

        Ok(())
    }
}

fn ordered_entries(options: &HashMap<String, String>) -> Vec<(&String, &String)> {
    let mut entries = options.iter().collect::<Vec<_>>();
    entries.sort_by(|a, b| a.0.cmp(b.0));
    entries
}

#[derive(Clone, Copy)]
struct ProfileSchema {
    name: &'static str,
    fields: &'static [ProfileField],
}

#[derive(Clone, Copy)]
struct ProfileField {
    name: &'static str,
    kind: FieldKind,
    required: bool,
}

#[derive(Clone, Copy)]
enum FieldKind {
    Text,
    Bool,
}

const fn field(name: &'static str) -> ProfileField {
    ProfileField {
        name,
        kind: FieldKind::Text,
        required: false,
    }
}

const fn required_field(name: &'static str) -> ProfileField {
    ProfileField {
        name,
        kind: FieldKind::Text,
        required: true,
    }
}

const fn bool_field(name: &'static str) -> ProfileField {
    ProfileField {
        name,
        kind: FieldKind::Bool,
        required: false,
    }
}

const SCHEMAS: &[ProfileSchema] = &[
    ProfileSchema {
        name: "azblob",
        fields: &[
            field("root"),
            required_field("container"),
            field("endpoint"),
            field("account_name"),
            field("account_key"),
            field("encryption_key"),
            field("encryption_key_sha256"),
            field("encryption_algorithm"),
            field("sas_token"),
            field("batch_max_operations"),
        ],
    },
    ProfileSchema {
        name: "azdls",
        fields: &[
            field("root"),
            required_field("filesystem"),
            field("endpoint"),
            field("account_name"),
            field("account_key"),
            field("client_secret"),
            field("tenant_id"),
            field("client_id"),
            field("sas_token"),
            field("authority_host"),
            bool_field("enable_hns"),
        ],
    },
    ProfileSchema {
        name: "azfile",
        fields: &[
            field("root"),
            field("endpoint"),
            required_field("share_name"),
            field("account_name"),
            field("account_key"),
            field("sas_token"),
        ],
    },
    ProfileSchema {
        name: "cos",
        fields: &[
            field("root"),
            field("endpoint"),
            field("secret_id"),
            field("secret_key"),
            field("bucket"),
            bool_field("enable_versioning"),
            bool_field("disable_config_load"),
        ],
    },
    ProfileSchema {
        name: "dropbox",
        fields: &[
            field("root"),
            field("access_token"),
            field("refresh_token"),
            field("client_id"),
            field("client_secret"),
        ],
    },
    ProfileSchema {
        name: "fs",
        fields: &[field("root"), field("atomic_write_dir")],
    },
    ProfileSchema {
        name: "gcs",
        fields: &[
            field("root"),
            required_field("bucket"),
            field("endpoint"),
            field("scope"),
            field("service_account"),
            field("credential"),
            field("credential_path"),
            field("predefined_acl"),
            field("default_storage_class"),
            bool_field("allow_anonymous"),
            bool_field("disable_vm_metadata"),
            bool_field("disable_config_load"),
            field("token"),
        ],
    },
    ProfileSchema {
        name: "ghac",
        fields: &[
            field("root"),
            field("version"),
            field("endpoint"),
            field("runtime_token"),
        ],
    },
    ProfileSchema {
        name: "http",
        fields: &[
            field("endpoint"),
            field("username"),
            field("password"),
            field("token"),
            field("root"),
        ],
    },
    ProfileSchema {
        name: "ipmfs",
        fields: &[field("root"), field("endpoint")],
    },
    ProfileSchema {
        name: "memory",
        fields: &[field("root")],
    },
    ProfileSchema {
        name: "obs",
        fields: &[
            field("root"),
            field("endpoint"),
            field("access_key_id"),
            field("secret_access_key"),
            field("bucket"),
            bool_field("enable_versioning"),
        ],
    },
    ProfileSchema {
        name: "oss",
        fields: &[
            field("root"),
            field("endpoint"),
            field("presign_endpoint"),
            required_field("bucket"),
            field("addressing_style"),
            field("presign_addressing_style"),
            bool_field("enable_versioning"),
            field("server_side_encryption"),
            field("server_side_encryption_key_id"),
            bool_field("allow_anonymous"),
            field("access_key_id"),
            field("access_key_secret"),
            field("security_token"),
            field("batch_max_operations"),
            field("delete_max_size"),
            field("role_arn"),
            field("role_session_name"),
            field("oidc_provider_arn"),
            field("oidc_token_file"),
            field("sts_endpoint"),
        ],
    },
    ProfileSchema {
        name: "s3",
        fields: &[
            field("root"),
            required_field("bucket"),
            bool_field("enable_versioning"),
            field("endpoint"),
            field("region"),
            field("access_key_id"),
            field("secret_access_key"),
            field("session_token"),
            field("role_arn"),
            field("external_id"),
            field("role_session_name"),
            bool_field("disable_config_load"),
            bool_field("disable_ec2_metadata"),
            bool_field("allow_anonymous"),
            field("server_side_encryption"),
            field("server_side_encryption_aws_kms_key_id"),
            field("server_side_encryption_customer_algorithm"),
            field("server_side_encryption_customer_key"),
            field("server_side_encryption_customer_key_md5"),
            field("default_storage_class"),
            bool_field("enable_virtual_host_style"),
            field("batch_max_operations"),
            field("delete_max_size"),
            bool_field("disable_stat_with_override"),
            field("checksum_algorithm"),
            bool_field("disable_write_with_if_match"),
            bool_field("enable_write_with_append"),
            bool_field("disable_list_objects_v2"),
            bool_field("enable_request_payer"),
            field("default_acl"),
        ],
    },
    ProfileSchema {
        name: "webdav",
        fields: &[
            field("endpoint"),
            field("username"),
            field("password"),
            field("token"),
            field("root"),
            bool_field("disable_copy"),
            bool_field("disable_create_dir"),
            bool_field("enable_user_metadata"),
            field("user_metadata_prefix"),
            field("user_metadata_uri"),
        ],
    },
    ProfileSchema {
        name: "webhdfs",
        fields: &[
            field("root"),
            field("endpoint"),
            field("user_name"),
            field("delegation"),
            bool_field("disable_list_batch"),
            field("atomic_write_dir"),
        ],
    },
];

fn schema_names() -> Vec<&'static str> {
    SCHEMAS.iter().map(|schema| schema.name).collect()
}

fn profile_schema(name: &str) -> Option<&'static ProfileSchema> {
    SCHEMAS.iter().find(|schema| schema.name == name)
}

fn ordered_prompt_fields(schema: &ProfileSchema) -> Vec<&ProfileField> {
    schema
        .fields
        .iter()
        .filter(|field| field.required)
        .chain(schema.fields.iter().filter(|field| !field.required))
        .collect()
}

fn prompt_required_text(message: &str) -> Result<Option<String>> {
    loop {
        match Text::new(message).prompt() {
            Ok(value) if value.trim().is_empty() => {
                println!("{message} is required.");
            }
            Ok(value) => return Ok(Some(value.trim().to_string())),
            Err(InquireError::OperationCanceled | InquireError::OperationInterrupted) => {
                return Ok(None);
            }
            Err(err) => return Err(err.into()),
        }
    }
}

fn prompt_field(field: &ProfileField) -> Result<Option<Option<String>>> {
    let message = if field.required {
        format!("{} (required)", field.name)
    } else {
        format!("{} (optional, leave empty to skip)", field.name)
    };

    match field.kind {
        FieldKind::Text if is_secret_like(field.name) => prompt_secret_field(field, &message),
        FieldKind::Text => prompt_text_field(field, &message),
        FieldKind::Bool => prompt_bool_field(field, &message),
    }
}

fn prompt_text_field(field: &ProfileField, message: &str) -> Result<Option<Option<String>>> {
    loop {
        match Text::new(message).prompt() {
            Ok(value) => {
                let value = value.trim().to_string();
                if field.required && value.is_empty() {
                    println!("{} is required.", field.name);
                    continue;
                }
                return Ok(Some((!value.is_empty()).then_some(value)));
            }
            Err(InquireError::OperationCanceled | InquireError::OperationInterrupted) => {
                return Ok(None);
            }
            Err(err) => return Err(err.into()),
        }
    }
}

fn prompt_secret_field(field: &ProfileField, message: &str) -> Result<Option<Option<String>>> {
    loop {
        match Password::new(message)
            .with_display_mode(PasswordDisplayMode::Masked)
            .without_confirmation()
            .prompt()
        {
            Ok(value) => {
                let value = value.trim().to_string();
                if field.required && value.is_empty() {
                    println!("{} is required.", field.name);
                    continue;
                }
                return Ok(Some((!value.is_empty()).then_some(value)));
            }
            Err(InquireError::OperationCanceled | InquireError::OperationInterrupted) => {
                return Ok(None);
            }
            Err(err) => return Err(err.into()),
        }
    }
}

fn prompt_bool_field(_field: &ProfileField, message: &str) -> Result<Option<Option<String>>> {
    match Confirm::new(message).with_default(false).prompt() {
        Ok(true) => Ok(Some(Some("true".to_string()))),
        Ok(false) => Ok(Some(None)),
        Err(InquireError::OperationCanceled | InquireError::OperationInterrupted) => Ok(None),
        Err(err) => Err(err.into()),
    }
}

fn is_secret_like(name: &str) -> bool {
    name.contains("secret")
        || name.contains("password")
        || name.contains("token")
        || name.contains("credential")
        || name.contains("delegation")
        || name.ends_with("_key")
        || name.contains("_key_")
        || name == "key"
}

fn config_profile_exists(config_path: &Path, profile_name: &str) -> Result<bool> {
    let root = load_config_table(config_path)?;
    let Some(profiles) = root.get("profiles") else {
        return Ok(false);
    };
    let profiles = profiles
        .as_table()
        .ok_or_else(|| anyhow!("`profiles` in {} must be a table", config_path.display()))?;
    Ok(profiles.contains_key(profile_name))
}

fn write_profile(
    config_path: &Path,
    profile_name: &str,
    profile: &HashMap<String, String>,
) -> Result<()> {
    let mut root = load_config_table(config_path)?;

    let profiles = root
        .entry("profiles".to_string())
        .or_insert_with(|| Value::Table(Table::new()))
        .as_table_mut()
        .ok_or_else(|| anyhow!("`profiles` in {} must be a table", config_path.display()))?;

    let mut profile_table = Table::new();
    for (key, value) in ordered_entries(profile) {
        profile_table.insert(key.clone(), Value::String(value.clone()));
    }
    profiles.insert(profile_name.to_string(), Value::Table(profile_table));

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create config directory {}", parent.display()))?;
    }

    let content = toml::to_string_pretty(&Value::Table(root))?;
    fs::write(config_path, content)
        .with_context(|| format!("failed to write config file {}", config_path.display()))?;

    Ok(())
}

fn load_config_table(config_path: &Path) -> Result<Table> {
    if !config_path.exists() {
        return Ok(Table::new());
    }

    let content = fs::read_to_string(config_path)
        .with_context(|| format!("failed to read config file {}", config_path.display()))?;
    if content.trim().is_empty() {
        return Ok(Table::new());
    }

    toml::from_str::<Table>(&content)
        .with_context(|| format!("failed to parse config file {}", config_path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_like_fields_are_detected() {
        assert!(is_secret_like("secret_access_key"));
        assert!(is_secret_like("account_key"));
        assert!(is_secret_like("session_token"));
        assert!(is_secret_like("password"));
        assert!(is_secret_like("credential_path"));
        assert!(!is_secret_like("endpoint"));
    }

    #[test]
    fn required_fields_are_prompted_first() {
        let schema = profile_schema("s3").expect("s3 schema should be present");
        let fields = ordered_prompt_fields(schema)
            .into_iter()
            .map(|field| field.name)
            .collect::<Vec<_>>();

        assert_eq!(fields[0], "bucket");
        assert_eq!(fields[1], "root");
        assert!(
            fields
                .iter()
                .skip_while(|name| schema
                    .fields
                    .iter()
                    .any(|field| field.name == **name && field.required))
                .all(|name| schema
                    .fields
                    .iter()
                    .any(|field| field.name == *name && !field.required))
        );
    }

    #[test]
    fn write_profile_creates_profiles_table() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let config_path = dir.path().join("config.toml");
        let profile = HashMap::from([
            ("type".to_string(), "s3".to_string()),
            ("bucket".to_string(), "example".to_string()),
            ("enable_virtual_host_style".to_string(), "true".to_string()),
        ]);

        write_profile(&config_path, "demo", &profile)?;

        let cfg = Config::load_from_file(&config_path)?;
        let stored = cfg.profile("demo").expect("profile should be present");
        assert_eq!(stored["type"], "s3");
        assert_eq!(stored["bucket"], "example");
        assert_eq!(stored["enable_virtual_host_style"], "true");
        Ok(())
    }

    #[test]
    fn write_profile_preserves_existing_profile() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let config_path = dir.path().join("config.toml");
        fs::write(
            &config_path,
            r#"
[profiles.existing]
type = "memory"
"#,
        )?;
        let profile = HashMap::from([
            ("type".to_string(), "fs".to_string()),
            ("root".to_string(), "/tmp".to_string()),
        ]);

        write_profile(&config_path, "local", &profile)?;

        let cfg = Config::load_from_file(&config_path)?;
        assert_eq!(cfg.profile("existing").unwrap()["type"], "memory");
        assert_eq!(cfg.profile("local").unwrap()["root"], "/tmp");
        Ok(())
    }
}
