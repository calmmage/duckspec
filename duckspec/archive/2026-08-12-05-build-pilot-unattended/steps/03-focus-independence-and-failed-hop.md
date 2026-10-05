# Focus independence and failed hop

Prove selection/focus does not stop hops; disarm + system notice when a decided hop cannot
send.

## Prerequisites

- [x] @step unattended-auto-send-policy

## Tasks

- [x] 1. Audit SelectChange / area switch: no cancel or disarm of background piloted
         sessions

- [x] 2. On failed pilot send (no handle / send blocked): disarm, system notice, clear
         pending hop

- [x] 3. @spec chat/build-pilot Focus independence: Select other change leaves armed pilot intact

- [x] 4. @spec chat/build-pilot Focus independence: Hop after TurnComplete does not require active session

- [x] 5. @spec chat/build-pilot Failed hop disarms: Failed hop with no handle disarms and notices
