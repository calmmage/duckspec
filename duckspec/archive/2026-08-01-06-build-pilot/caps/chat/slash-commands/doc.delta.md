# @ Chat slash commands

Kinded slash-command catalog for chat completion, local system handlers (including `/help`
and build-pilot commands), and a double-slash escape so colliding agent skills stay
reachable.

## ~ Kinds

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

## ~ Submit routing

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
