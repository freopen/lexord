# Dev Container

This container is intentionally locked down for day-to-day Rust development:

- runs as the non-root `vscode` user
- drops Linux capabilities and enables `no-new-privileges`
- limits processes
- keeps Codex state in a named volume

The image starts from the Debian Trixie devcontainer base, installs Debian's
`rustup` package, then installs the nightly Rust toolchain, `clippy`, `rustfmt`,
`cargo-make`, `cargo-audit`, and `cargo-afl` as the `vscode` user. It also
includes LLVM/Clang and `tmux`.

Codex state is stored in the project-specific `lexord-codex` volume mounted at
`/home/vscode/.codex`.

## Notes

Codex is intended to run in Full Access mode inside this container. The container
is treated as the security boundary for dependency execution, while Codex's state
is isolated from the host by the project-specific volume.

The Rust toolchain and Cargo-installed tools are owned by `vscode` so
user-specific tool caches, such as the afl.rs-managed AFL++ build, stay visible
to the same user that runs the tools.

Native LLDB debugging may require adding `SYS_PTRACE`, depending on the host
container runtime.
