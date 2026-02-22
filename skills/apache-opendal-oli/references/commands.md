# OLI Command Reference

## Location Semantics

- Use local filesystem mode when the argument does not contain `:/`.
- Use remote profile mode when the argument is `<profile>:/<path>`.
- Avoid `://` in location strings. `oli` treats host parts as invalid.

Examples:

```bash
# local path
oli ls './data/'

# remote path
oli ls 's3:/data/'

# invalid syntax (do not use)
oli ls 's3://data/'
```

## Config Resolution

- Default config path:
  - Linux: `~/.config/oli/config.toml`
  - macOS: `~/Library/Application Support/oli/config.toml`
  - Windows: `%APPDATA%\oli\config.toml`
- Environment variables override file configuration.
- Environment key format: `OLI_PROFILE_<PROFILE>_<OPTION>`

Examples:

```bash
OLI_PROFILE_S3_TYPE=s3
OLI_PROFILE_S3_BUCKET=my-bucket
OLI_PROFILE_S3_REGION=us-east-1
```

## Global Parameter

`--config <path>` is available through `ConfigParams` in command implementations.

```bash
oli --config ./config.toml ls 's3:/'
oli cp --config ./config.toml './a.txt' 's3:/a.txt'
```

## Commands

### Read-Only

- `oli ls <target>`
- `oli ls -r <target>`
- `oli ls -T <target>`
- `oli cat <target>`
- `oli stat <target>`
- `oli config view`

### Mutating

- `oli cp <source> <destination>`
- `oli cp -r <source_dir> <destination_dir>`
- `oli cp --content-type application/json <source> <destination>`
- `oli mv <source> <destination>`
- `oli mv -r <source_dir> <destination_dir>`
- `oli rm <target>`
- `oli rm -r <target_dir>`
- `oli tee <destination>`
- `oli tee -a <destination>`
- `oli edit <target>`

### Benchmark

- `oli bench <profile> <suite.toml>`

## Binary Aliases

`oli` supports executable-name aliases:

- `obench` -> `bench`
- `ocat` -> `cat`
- `ocp` -> `cp`
- `ols` -> `ls`
- `orm` -> `rm`
- `ostat` -> `stat`
- `omv` -> `mv`

Prefer `oli <subcommand>` for clarity unless alias behavior is explicitly required.
