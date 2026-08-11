# System slash build pilot commands

Register `/build-auto` and `/build-fast`, parse optional args into local build-pilot
classification, and surface them as System in the completion catalog.

## Tasks

- [x] 1. Extend `system_registry` with `build-auto` and `build-fast`

- [x] 2. Extend `SubmitSlash` and `parse_submit_slash` for `/build-auto|fast` [args] →
         `LocalBuildPilot`

- [x] 3. @spec chat/slash-commands Kinded completion catalog: build-auto and build-fast are System

- [x] 4. @spec chat/slash-commands Build pilot system classification: build pilot names are system registry commands

- [x] 5. @spec chat/slash-commands Build pilot system classification: build-auto with args classifies as local build pilot

- [x] 6. @spec chat/slash-commands Build pilot system classification: bare build-fast classifies as local build pilot
