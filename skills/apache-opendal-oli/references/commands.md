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

## Profile Setup

For profile source modes, config examples, and `--config` usage rules, read:

- `references/profile-setup.md`

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
