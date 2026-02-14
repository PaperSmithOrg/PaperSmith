# PaperSmith

<!--toc:start-->

- [PaperSmith](#papersmith)
  - [Main Contributors](#main-contributors)
  - [Roadmap](#roadmap)
    - [Done](#done)
    - [Planned](#planned)
    - [Future ideas](#future-ideas)
  - [Installation](#installation)
  - [Build from source](#build-from-source)
  <!--toc:end-->

_A free and open-source writing application for authors_

> [!CAUTION]
> This project is in its early stages of development and we consider it unstable. Use at your own risk.

## Main Contributors

- Toll25
- T-Ricstar
- Alllpacka
- DotDo1

## Roadmap

Features that are either already finished, in progress or planned for future development. It also includes ideas for later implementation.

### Done

- [x] Split-View
- [x] Project Explorer
- [x] Load Project
- [x] Saving files
- [x] Statistics
- [x] Project creation wizard
- [x] Markdown Formatting
- [x] Autosaving
- [x] Settings menu
- [x] Statistics Window

### Planned

- [ ] Automatic Backups
- [ ] Export options
- [ ] Spellcheck
- [ ] Single-View

### Future ideas

- [ ] Multiple open documents
- [ ] Page-full layout
- [ ] Grammar check

## Installation

Binaries are available from Github releases.

Not yet packaged anywhere.

## Build from source

1. Install Rust
2. Install [Tauri V1 dependencies](https://v1.tauri.app/v1/guides/getting-started/prerequisites#1-system-dependencies)
3. Clone repo `git clone PaperSmithOrg/PaperSmith && cd PaperSmith`
4. Run `make install` (installs trunk, tauri-cli, npm deps and adds WebAssembly target to rustup)
5. Run `make dev` / `make build`
