# Session sharing

Implement drive-versus-display write policy, external reload, in-flight shield, and
last-write-wins on shared session files via duckcore helpers consumed by the UIs.

## Prerequisites

- [x] @step extract-duckcore

## Tasks

- [x] 1. Model drive versus display for loaded sessions; gate persist so only driven
         sessions write

- [x] 2. Watch session paths via the duckcore watcher; reload idle displayed sessions on
         external modify or remove

- [x] 3. Shield in-flight local turns from external reloads until settle, then resume idle
         reload

- [x] 4. Assert no multi-process locks; concurrent writes remain last atomic write wins

- [x] 5. @spec chat/session-sharing Drive-only writes: Displayed-only session is never written

- [x] 6. @spec chat/session-sharing Drive-only writes: Completing a local turn writes the session

- [x] 7. @spec chat/session-sharing External change reload: External write reloads a displayed idle session

- [x] 8. @spec chat/session-sharing External change reload: External removal drops a displayed idle session

- [x] 9. @spec chat/session-sharing In-flight external shield: External change during a local turn does not replace mid-turn state

- [x] 10. @spec chat/session-sharing In-flight external shield: After local turn settlement, a later external change reloads

- [x] 11. @spec chat/session-sharing Concurrent drive is last-write-wins: Later write is the persisted content
