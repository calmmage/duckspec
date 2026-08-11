# Install quit-before-deploy warning

Print quit guidance at the start of `just install` before Applications replace so a
running app does not fail mid-copy.

## Prerequisites

- [x] @step local-install-applications

## Context

Review finding 2 (fidelity/minor): design/plaque say quit first, but the install recipe
only printed that after `rm`/`cp`. Keep the final reopen note.

## Tasks

- [x] 1. Echo quit-before-replace near the top of the root `justfile` `install` recipe
         (before bundle/deploy)

- [x] 2. Confirm `install_recipe_deploys_cargo_bins_and_applications` still passes
