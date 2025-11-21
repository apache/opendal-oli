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

use anyhow::Result;
use inquire::Select;
use inquire::error::InquireError;

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
    View(ConfigViewCmd),
}

impl ConfigCmd {
    pub fn run(self) -> Result<()> {
        match self.subcommand {
            ConfigSubcommand::View(cmd) => cmd.run(),
        }
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
