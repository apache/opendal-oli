# OLI Safety And Troubleshooting

## Safe Execution Checklist

1. Resolve intent:
- `read-only` for inspection
- `mutating` for data changes

2. Validate source and destination paths with read-only checks:

```bash
oli ls 's3:/path/'
oli stat 's3:/path/to/object'
```

3. Execute mutation only after verification:

```bash
oli cp 's3:/path/to/object' 'r2:/backup/object'
```

4. Verify result:

```bash
oli stat 'r2:/backup/object'
```

## High-Risk Operations

### Recursive Delete

- Always inspect scope before `rm -r`.

```bash
oli ls -r 's3:/staging/prefix/'
oli rm -r 's3:/staging/prefix/'
oli ls 's3:/staging/'
```

### Recursive Move

- `mv -r` moves data and deletes source entries afterward.
- Validate source and destination roots before running.

### Recursive Copy

- Use `cp -r` only for directory-level workflows.
- Confirm destination is a directory or a new path intended as directory root.

## Common Errors And Fixes

### `unknown profile: <name>`

Cause:
- Profile name missing in config and environment.

Fix:
- Add `[profiles.<name>]` in `config.toml`, or set `OLI_PROFILE_<NAME>_TYPE`.
- Use `oli config view` to inspect available profiles.

### `missing 'type' in profile`

Cause:
- Profile exists but has no `type` key.

Fix:
- Add `type = "<service>"` in the profile table.

### `Host part in a location is not supported`

Cause:
- Used `://` instead of `:/`.

Fix:
- Replace `s3://a/b` with `s3:/a/b`.

### `can not move a directory in non-recursive mode`

Cause:
- Tried to move directory with `mv` without `-r`.

Fix:
- Use `mv -r`.

### `Recursive copy destination '...' exists but is not a directory`

Cause:
- `cp -r` destination points to an existing file.

Fix:
- Use a directory destination path.

### `No configuration profiles found in ...`

Cause:
- Config file missing or empty at the selected `--config` path.

Fix:
- Create config file with at least one profile.

### `TOML parse error ... missing field profiles`

Cause:
- `--config` points to an invalid or empty TOML file.
- In environment-variable mode, `--config` was passed unintentionally.

Fix:
- In config-file mode, pass a valid config file containing `[profiles.*]`.
- In environment-variable mode, do not pass `--config`.

## Operational Patterns

### Cross-Profile Transfer

```bash
oli cp 's3:/dataset/part-0001.parquet' 'r2:/archive/part-0001.parquet'
oli stat 'r2:/archive/part-0001.parquet'
```

### Controlled Metadata Check

```bash
oli stat 's3:/dataset/part-0001.parquet'
```

Use the output to validate:
- `size`
- `etag` (if available)
- `content-type`
- `last-modified`
