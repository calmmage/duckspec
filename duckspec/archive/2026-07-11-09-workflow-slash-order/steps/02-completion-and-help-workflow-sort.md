# Completion and help Workflow sort

Sort Workflow by order key (unknown last) in `filter_commands` and the `/help` Workflow
section.

## Prerequisites

- [x] @step order-field-and-frontmatter-discovery

## Tasks

- [x] 1. Add `slash_order_rank` (or equivalent) and use it in `filter_commands` after kind
         rank: score ↓, kind, order_key (Some ascending, None last), name

- [x] 2. Sort Workflow entries in `append_kind_section` / `build_system_help_body` by
         order_key then name; leave System and Agent name-sorted

- [x] 3. @spec chat/slash-commands Kind cues in completion: Equal scores order Workflow by order key then name

- [x] 4. @spec chat/slash-commands Kind cues in completion: Workflow without order key sorts after ordered Workflow

- [x] 5. @spec chat/slash-commands Local system submit: Help Workflow section lists by order key then name
