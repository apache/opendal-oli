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

# Security Policy

## Reporting a Vulnerability

Apache OpenDAL Oli follows the
[Apache Software Foundation security process](https://www.apache.org/security/).

Please report suspected vulnerabilities privately to `security@apache.org` or
`private@opendal.apache.org` before public disclosure. Do not open public GitHub
issues or pull requests for security reports.

Please include:

- the project name, `Apache OpenDAL Oli`;
- affected version, commit, platform, and installation method;
- the command, profile shape, service type, and signing mode involved;
- whether credentials, signed requests, local files, object data, or metadata
  can be exposed or modified;
- a minimal reproducer and any relevant logs with secrets redacted.

## Supported Use

`oli` is a local command-line client for OpenDAL-backed storage. It runs with
the authority of the local user account and the credentials configured in its
profile, environment, ambient credential chain, or custom signing setup. For
services that require request signing, signing and credential lookup are
delegated to OpenDAL and its signing integrations.

Users should:

- protect `~/.config/oli/config.toml` and equivalent platform config files as
  secret material when they contain credentials;
- protect environment variables named like `OLI_PROFILE_<PROFILE>_<OPTION>`;
- verify storage endpoints before using profiles that sign requests;
- avoid placing long-lived credentials in shell history, terminal transcripts,
  CI logs, or shared config files;
- treat `oli config view`, command output, object paths, metadata, benchmark
  reports, and error messages as potentially sensitive local output;
- use least-privilege storage credentials and service-side access controls.

## Threat Model

The security boundary, signing assumptions, in-scope issues, out-of-scope
deployment responsibilities, and triage guidance are documented in
[THREAT_MODEL.md](./THREAT_MODEL.md).
