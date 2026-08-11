# Chat slash commands

Kinded slash-command catalog for chat completion, local system handlers (including
`/help`), and a double-slash escape so colliding agent skills stay reachable.

Kinded slash-command catalog for chat completion, local system handlers (including `/help`
and build-pilot commands), and a double-slash escape so colliding agent skills stay
reachable.

## Kinds

Every completion entry has one kind:

```
| Kind     | Meaning                                                         | Example        |
| -------- | --------------------------------------------------------------- | -------------- |
| System   | Recognized by duckboard first (local handler and/or local arm)  | `/help`        |
| Workflow | Duckspec stage templates; sent to the agent                     | `/ds-spec`     |
| Agent    | Harness / project / plugin skills; sent as-is                   | `/review`      |
```

The catalog is the merge of a duckboard system registry and harness discovery. On a name
collision, System wins — one entry, kind System.

Claude interactive builtins (`clear`, `compact`, `cost`, `help`, `model`) are not injected
as Agent entries by shared discovery. System names (`help`, `build-auto`, `build-fast`)
come only from the duckboard registry.

## Submit routing

```
submit text
    │
    ├── bare /help              → local help (no agent turn)
    ├── /build-auto|fast [args] → local build-pilot classification
    │                              (arm + kick owned by build-pilot)
    ├── bare //name             → agent turn, prompt = /name, user text = //name
    └── anything else           → normal agent turn
```

System registry names: `help`, `build-auto`, `build-fast`. Build-pilot submits may include
free-text arguments after the command token; help remains bare-only for its local path.

## Local `/help`

Records a user message (`/help`) then a system message. Does not stream, prime, or consume
selection attachments.

System message shape:

1. Fixed prefix: running system command `/help`; agent help via `//help`
2. Sections from the live catalog by kind (System, Workflow, Agent) — omit empty sections
3. Escape note for `//help`

Within the Workflow section, entries list by ascending order key, then name; entries
without an order key follow ordered ones. Agent section titles include the active harness
id when present (e.g. Agent skills → grok).

## Completion cues

```
| Kind     | Name color     | Tag   |
| -------- | -------------- | ----- |
| System   | system accent  | `sys` |
| Workflow | workflow color | —     |
| Agent    | agent color    | —     |
```

The three name colors are pairwise distinct. Fuzzy score still ranks matches. Equal scores
break ties as System → Workflow → Agent. Among Workflow entries with equal scores, order
keys from command frontmatter sort ascending; entries with no order key come after ordered
ones; equal keys break ties by name. The popup does not show stage numbers.

## Workflow frontmatter

Installable duckspec command files may declare YAML frontmatter used when discovery builds
catalog entries:

```
| Field         | Role                                              |
| ------------- | ------------------------------------------------- |
| `description` | Short blurb on the completion row and in `/help`  |
| `order`       | Sort key among Workflow entries (not shown in UI) |
```

Accepted `order` forms: a non-negative integer (`1`) or an integer with exactly one
decimal digit (`3.1`). Values are stored as tenths for sorting (`1` → 10, `3.1` → 31). Any
other form is ignored (no order key). Missing `description` yields an empty blurb; missing
or invalid `order` yields no key and sorts after ordered Workflow peers.
