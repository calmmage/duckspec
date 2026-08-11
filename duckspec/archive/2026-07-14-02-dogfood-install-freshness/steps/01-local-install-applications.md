# Local install applications

Extend `just install` so it bundles and deploys `/Applications/Duckboard.app` after cargo
install, and lock the recipe shape with a code-backed scenario.

## Tasks

- [x] 1. Update root `justfile` `install` to run bundle assembly and replace
         `/Applications/Duckboard.app`

- [x] 2. @spec shell/local-install Cargo bins and Applications app: Install recipe deploys cargo bins and Applications
