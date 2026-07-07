<!--
  Licensed to the Apache Software Foundation (ASF) under one
  or more contributor license agreements.  See the NOTICE file
  distributed with this work for additional information
  regarding copyright ownership.  The ASF licenses this file
  to you under the Apache License, Version 2.0 (the
  "License"); you may not use this file except in compliance
  with the License.  You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

  Unless required by applicable law or agreed to in writing,
  software distributed under the License is distributed on an
  "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
  KIND, either express or implied.  See the License for the
  specific language governing permissions and limitations
  under the License.
-->

# Apache OpenDAL Oli Threat Model

## 1. Status

This document defines the security boundary for Apache OpenDAL Oli (`oli`), a
local command-line interface for manipulating data through Apache OpenDAL
profiles. Use it to decide whether a report belongs to `oli`, OpenDAL or a
signing component, a storage backend, or the user's deployment.

The canonical disclosure process is in [SECURITY.md](./SECURITY.md). Reports
that may affect security should be reported privately before public disclosure.

## 2. Purpose

`oli` is a local CLI. It parses command-line arguments, loads storage profiles,
constructs OpenDAL `Operator` instances, and runs file-like operations such as
`ls`, `cat`, `cp`, `mv`, `rm`, `tee`, `edit`, `stat`, and `bench`.

The most security-sensitive path is request signing. `oli` does not implement a
general key-management system itself; it passes profile options and operation
inputs into OpenDAL, which may use service-specific signing integrations and
custom key providers to sign requests.

`oli` is not a daemon, gateway, identity provider, authorization service,
sandbox, key-management service, policy engine, malware scanner, or
multi-tenant broker. It does not decide whether a human, script, tenant, or CI
job is authorized to use the configured credentials.

The most important premises are:

> `oli` trusts the local user, selected config file, selected environment, and
> selected storage endpoint as intentional inputs.
>
> `oli` must still preserve its own CLI boundary: it must not mix credentials
> between profiles, sign requests for the wrong endpoint, leak secrets through
> unintended output, or map local paths contrary to the documented operation.

## 3. System Model

An `oli` deployment has these participants:

- Local user or script: Trusted invoker that chooses command arguments, config
  path, profile, stdin, stdout and stderr handling, environment, and editor.
- `oli` process: CLI that loads profile data, parses locations, creates
  OpenDAL operators, streams data, prints output, and exits.
- OpenDAL library: In-process storage abstraction used by `oli` to implement
  service-specific operations, credential loading, request construction, and
  signing.
- Custom request signer or key provider: Code or configuration used through
  OpenDAL to load credentials or sign requests for services that require
  request signing.
- Storage backend: Service selected by the trusted profile configuration, such
  as S3-compatible object storage, Azure, GCS, HTTP, WebDAV, WebHDFS, or local
  filesystem.
- Local editor: Program selected by `$EDITOR` and invoked by `oli edit` with
  object contents in a temporary file.
- Local OS account and filesystem: Provides config files, environment
  variables, temp directories, local paths, process permissions, and terminal
  behavior.
- Network and proxy environment: Carries requests to the configured storage
  endpoint. TLS, proxies, DNS, and CA policy are primarily provided by OpenDAL
  dependencies and the host environment.

## 4. Assets

The assets in scope are:

- static access keys, secret keys, session tokens, bearer tokens, private key
  material, service-account files, and custom signing keys;
- signed requests, canonical request inputs, signature headers, query
  signatures, and presigned URLs if exposed by a future command;
- profile configuration from config files and `OLI_PROFILE_<PROFILE>_<OPTION>`
  environment variables;
- object bytes, object metadata, paths, directory names, roots, bucket names,
  endpoints, and namespaces;
- local files read or written by filesystem locations;
- temporary files created for `oli edit`;
- stdout, stderr, progress output, error chains, terminal transcripts, CI logs,
  and shell history;
- remote storage state modified by `cp`, `mv`, `rm`, `tee`, `edit`, and
  `bench`.

## 5. Trust Boundaries

### 5.1 Command Line and Stdio

Command-line arguments, stdin, stdout, and stderr are controlled by the local
user or calling script. `oli` may display paths, metadata, benchmark paths,
progress information, and object bytes because those are normal command
results.

Reports are in scope when `oli` prints credential material, signing secrets, or
unexpectedly sensitive profile values outside commands whose purpose is to show
configuration.

### 5.2 Config Files and Environment

`oli` loads profile configuration from the selected config file and overlays
matching environment variables. Environment variables have higher precedence
than the config file.

The local user is responsible for protecting profile files and process
environment. A local attacker who can edit the config file, control
`--config`, or inject environment variables can redirect endpoints, change
roots, change credential options, enable alternate credential sources, or cause
requests to be signed with attacker-chosen settings.

`oli` is responsible for applying the documented precedence consistently and
for not leaking values that are credentials except through explicit local-admin
inspection commands.

### 5.3 Profiles, Endpoints, and Signing

A profile is an authority boundary. Commands using profile `A` must use only
profile `A` configuration, credentials, ambient credential chain, and custom
signing setup. A command using profile `B` must not inherit secrets or signer
state from profile `A` unless OpenDAL explicitly documents shared state selected
by the user.

When a profile signs requests, the signed host, endpoint, path, headers, query,
method, body hash, timestamp, expiry, and credential scope must match the
selected service's contract. `oli` delegates service-specific signing to
OpenDAL and its signing integrations, but reports are still in scope for this
repository if `oli` passes the wrong profile, path, endpoint, or operation into
OpenDAL.

If the user configures an attacker-controlled endpoint and gives it signing
credentials, sending signed requests to that endpoint is user configuration,
not an `oli` vulnerability. If `oli` ignores the configured endpoint or mixes it
with another profile's credentials, that is in scope.

### 5.4 Locations and Local Filesystem

Locations in the form `<profile>:/<path>` use the named profile. Inputs without
`:/` are treated as local filesystem locations through OpenDAL's filesystem
service.

`oli` must parse locations deterministically and reject unsupported host syntax
such as `<profile>://host/path`. For local filesystem locations, `oli` runs with
the local user's filesystem permissions. It is not a sandbox against the local
user's own paths, existing symlinks, mount points, file permissions, or
platform-specific filesystem behavior.

Reports are in scope when `oli` maps a user-provided local path to a different
location than its parser contract implies, or when a remote path is sent to a
different profile than the one selected in the command.

### 5.5 `oli edit`

`oli edit` downloads object contents into a local temporary file, launches
`$EDITOR`, reads the edited file, and uploads changed content.

The selected editor, editor plugins, terminal, temp directory, swap files,
backup files, and local OS account are part of the user's trusted computing
base. `oli` is responsible for using a secure temporary-file primitive and for
not leaving temporary files behind on successful upload. If upload fails, `oli`
may preserve the edited file so the user does not lose changes; that preserved
file is sensitive local data.

### 5.6 Destructive and High-Cost Commands

`rm`, recursive `rm`, `mv`, `cp`, `tee`, `edit`, and `bench` intentionally
modify storage state. `bench` intentionally writes benchmark objects, may use
parallelism, and may consume network, storage, and provider quota.

The local user is responsible for choosing profiles, paths, benchmark sizes,
parallelism, and retention options. Reports are in scope when `oli` modifies a
path other than the selected operation's documented target, crosses profile
boundaries, or fails to respect recursive versus non-recursive behavior.

## 6. In-Scope Security Properties

### 6.1 Credential and Key Isolation

`oli` must not leak or reuse credentials, key material, custom signer state, or
ambient credential results across independent profiles or commands.

Examples:

- `oli cp profile-a:/x profile-b:/y` must read with profile `A` and write with
  profile `B`; profile `A` credentials must not sign profile `B` requests.
- Environment overrides for one profile must not apply to another profile.
- A custom signer configured for one service must not be used for another
  service unless the user explicitly configures that sharing.

### 6.2 Signing Correctness at the CLI Boundary

`oli` must pass the intended operation, path, endpoint, profile data, and
headers into OpenDAL. Request signing bugs inside OpenDAL or its signing
subprojects should be reported against the component that owns the signer, but
`oli` remains responsible for errors introduced by CLI parsing, profile merge,
or command construction.

Examples:

- A destination upload must not be signed as a read operation.
- Recursive copy must not construct destination paths that escape the selected
  destination prefix.
- A content-type override must not be silently applied to the wrong destination
  object.

### 6.3 Secret Redaction and Local Output

`oli` must avoid exposing credentials through unintended debug output, error
contexts, panic messages, progress output, and benchmark reports.

`oli config view` is an explicit local inspection command and can reveal
profile values to the local user. Users should treat its output as secret when
profiles contain credentials. Future changes that make config inspection safer
by default are hardening improvements, not a reduction of the current threat
model.

Paths, roots, endpoints, bucket names, object metadata, ETags, and sizes can be
sensitive in some deployments, but they are normal operational output for a CLI.
Deployments that treat them as confidential should control terminal logging and
CI output.

### 6.4 Path and Profile Binding

`oli` must bind each command argument to the intended operator and object path.
It must not accidentally swap source and destination profiles, interpret remote
locations as local files, or treat unsupported URL host syntax as a valid
profile location.

### 6.5 Local Temporary Files

Temporary files created by `oli` must use OS-provided secure temporary-file
mechanisms. They must not use predictable names, follow attacker-controlled
symlink paths, or overwrite unrelated files through `oli` path construction.

### 6.6 Robustness

`oli` must handle untrusted backend responses, malformed metadata, unexpected
object names, and service errors without memory-safety violations, panics that
expose secrets, or inconsistent local cleanup. Semantic correctness of backend
data remains the backend's responsibility.

## 7. Out of Scope

The following are not `oli` vulnerabilities by default:

- A user configures malicious credentials, an attacker-controlled endpoint, or
  a profile root that points at sensitive storage.
- A local attacker can edit the selected config file, control the process
  environment, replace the `oli` binary, change `$EDITOR`, or read the user's
  terminal output.
- A storage backend lies about object bytes, metadata, ETags, timestamps,
  listing order, authorization, or durability.
- A provider-side bucket policy, IAM policy, OAuth scope, database permission,
  or filesystem permission is too broad.
- A script forwards untrusted tenant input into `oli` without its own
  authentication, authorization, validation, or quoting.
- A command intentionally prints object bytes, metadata, paths, or config values
  to a terminal or log chosen by the user.
- A user runs large recursive copies, recursive deletes, or benchmark workloads
  that consume quota, bandwidth, storage, or memory.
- TLS certificate policy, proxy configuration, DNS behavior, and CA bundles
  selected by the host environment, unless `oli` introduces a separate option
  that weakens them contrary to documentation.
- General dependency freshness, CI hardening, release infrastructure, and ASF
  infrastructure policy, unless a separate project policy says otherwise.

## 8. Triage Dispositions

Use these labels consistently:

- `VALID`: The report shows a reachable violation of an in-scope `oli`
  boundary, such as credential disclosure, cross-profile credential reuse,
  wrong-path mutation, unsafe temp-file behavior, or CLI-induced signing error.
- `VALID-UPSTREAM`: The report affects OpenDAL or a signing subproject rather
  than `oli`'s CLI boundary. Coordinate disclosure and fix in the owning
  component.
- `VALID-HARDENING`: There is no clear boundary violation, but behavior is
  risky or surprising enough that maintainers choose to harden docs, defaults,
  warnings, or redaction.
- `OUT-OF-SCOPE: local-control`: The report requires control of the local user
  account, config file, environment, editor, terminal, binary, or process
  supervisor.
- `OUT-OF-SCOPE: caller-config`: The report depends on a trusted user selecting
  a malicious endpoint, credential, root, signer, config path, or service
  option.
- `OUT-OF-SCOPE: backend`: The report depends on a configured backend being
  malicious, compromised, semantically wrong, or misconfigured.
- `OUT-OF-SCOPE: caller-authz`: The report depends on another application or
  script passing unauthorized user input into `oli`.
- `OUT-OF-SCOPE: expected-output`: The report is about output that the selected
  command is designed to display to the local user.
- `BY-DESIGN: property-not-provided`: The report asks `oli` to provide a
  property explicitly not provided here, such as multi-tenant authorization,
  endpoint trust decisions, backend-byte authentication, or default quota
  control.
- `MODEL-GAP`: The report cannot be classified by this document. Treat this as
  evidence that the model needs revision.

## 9. Examples

These examples describe classes of reports, not a complete list of affected
commands or services.

### 9.1 In Scope

- `oli cp a:/secret b:/copy` signs the destination write with profile `a`
  credentials instead of profile `b` credentials.
- `OLI_PROFILE_A_SECRET_ACCESS_KEY` unexpectedly changes any profile other than
  profile `a`.
- A malformed profile or backend response causes a panic that prints secret key
  material in an error chain.
- `oli edit` creates a predictable temporary path that another local user can
  pre-create as a symlink.
- Recursive copy constructs destination paths outside the selected destination
  prefix.
- A CLI parsing bug causes `profile:/path` to be sent to a different profile.

### 9.2 Out of Scope By Default

- A user stores long-lived cloud keys in `config.toml` with world-readable file
  permissions.
- A user sets `$EDITOR` to a program that exfiltrates temporary file contents.
- A user configures an attacker-controlled S3-compatible endpoint and sends it
  signed requests.
- A storage provider accepts an overprivileged key and permits destructive
  deletes.
- `oli cat` prints sensitive object bytes to a terminal transcript.
- A CI job runs `oli config view` and publishes the job log.
- A benchmark workload consumes provider quota.

## 10. Maintainer Decisions

- Relation to OpenDAL: `oli` inherits OpenDAL service behavior and threat
  boundaries where it delegates operation execution. CLI parsing, profile
  merge, command semantics, local output, and temp files remain `oli`
  responsibilities.
- Custom signing: Custom signers and key providers are trusted when configured
  by the user. `oli` must not cross-wire them between profiles or pass incorrect
  request inputs into OpenDAL.
- Config precedence: Environment profile variables override config-file profile
  values. This is intentional and must be documented because environment
  injection changes command authority.
- Local filesystem: Local paths run with the local user's OS permissions. `oli`
  is not a filesystem sandbox.
- Config inspection: `oli config view` is a local inspection command. Its output
  is sensitive when profile values are sensitive.
- Backend trust: The configured backend is trusted for object bytes, metadata,
  authorization decisions, consistency, and durability.
- Object integrity: `oli` does not provide end-to-end cryptographic object
  authentication above the configured backend. Users that require it must add it
  outside `oli`.
- Resource limits: `oli` does not provide default protection against
  intentionally large reads, listings, copies, deletes, or benchmark workloads.

## 11. Revision Triggers

Update this threat model when any of the following changes:

- `oli` adds commands for presigning, key management, credential generation, or
  custom signer configuration;
- config precedence, profile naming, or environment variable parsing changes;
- commands add new local filesystem write surfaces or long-lived local caches;
- `oli edit` changes temp-file preservation or editor invocation behavior;
- request-signing behavior becomes directly implemented in this repository;
- remote HTTP client, TLS, proxy, or endpoint policy becomes configurable by
  `oli`;
- output redaction behavior changes;
- a vulnerability report is classified as `MODEL-GAP`.
