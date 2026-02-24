# OLI Profile Setup

## Overview

Use one profile source mode per shell/session:

- Config-file mode: define profiles in TOML and pass `--config`.
- Environment-variable mode: define profiles in `OLI_PROFILE_*` and do not pass `--config`.

## Config Resolution

- Default config path:
  - Linux: `~/.config/oli/config.toml`
  - macOS: `~/Library/Application Support/oli/config.toml`
  - Windows: `%APPDATA%\oli\config.toml`
- Environment variables override file configuration.
- Environment key format: `OLI_PROFILE_<PROFILE>_<OPTION>`

## Mode A: Config-File Profiles

Use `--config` as a subcommand option (not a top-level global option).

Example `config.toml` with `s3` and `r2`:

```toml
[profiles.s3]
type = "s3"
bucket = "my-bucket"
region = "us-east-1"
endpoint = "https://s3.amazonaws.com"
access_key_id = "..."
secret_access_key = "..."
root = "/workspace-s3"

[profiles.r2]
type = "s3"
bucket = "my-bucket"
region = "auto"
endpoint = "https://<account_id>.eu.r2.cloudflarestorage.com"
access_key_id = "..."
secret_access_key = "..."
root = "/workspace-r2"
```

Run:

```bash
oli ls --config ./config.toml 's3:/'
oli cp --config ./config.toml 's3:/a.txt' 'r2:/a.txt'
```

## Mode B: Environment Variable Profiles

Define both profiles with `OLI_PROFILE_*`, then run commands without `--config`.

```bash
export OLI_PROFILE_S3_TYPE=s3
export OLI_PROFILE_S3_BUCKET=my-bucket
export OLI_PROFILE_S3_REGION=us-east-1
export OLI_PROFILE_S3_ENDPOINT=https://s3.amazonaws.com
export OLI_PROFILE_S3_ACCESS_KEY_ID=...
export OLI_PROFILE_S3_SECRET_ACCESS_KEY=...
export OLI_PROFILE_S3_ROOT=/workspace-s3

export OLI_PROFILE_R2_TYPE=s3
export OLI_PROFILE_R2_BUCKET=my-bucket
export OLI_PROFILE_R2_REGION=auto
export OLI_PROFILE_R2_ENDPOINT=https://<account_id>.eu.r2.cloudflarestorage.com
export OLI_PROFILE_R2_ACCESS_KEY_ID=...
export OLI_PROFILE_R2_SECRET_ACCESS_KEY=...
export OLI_PROFILE_R2_ROOT=/workspace-r2
```

Run:

```bash
oli ls 's3:/'
oli cp 's3:/data.txt' 'r2:/backup/data.txt'
```

## Common Pitfalls

- Do not use `://` in locations; use `profile:/path`.
- In environment-variable mode, do not pass `--config`.
- In config-file mode, pass a valid TOML file with `[profiles.*]`.
