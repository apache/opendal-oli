---
name: New Release
about: Use this template for start making a new release
title: "Tracking issues of Apache OpenDAL Oli (opendal-oli) ${opendal_oli_version} Release"
---

This issue is used to track tasks of the Apache OpenDAL Oli (opendal-oli) ${opendal_oli_version} release.

## Tasks

### Blockers

<!-- Blockers are the tasks that must be completed before the release. -->

### Build Release

#### GitHub Side

- [ ] Bump version in project
- [ ] Update docs
- [ ] Generate dependencies list
- [ ] Push release candidate tag to GitHub

#### ASF Side

- [ ] Create an ASF Release
- [ ] Upload artifacts to the SVN dist repo
- [ ] Close the Nexus staging repo

### Voting

- [ ] Start VOTE at the Apache OpenDAL community

### Official Release

- [ ] Push the release git tag
- [ ] Publish artifacts to SVN RELEASE branch
- [ ] Release Maven artifacts
- [ ] Send the announcement

For details of each step, please refer to: https://opendal.apache.org/community/release/
