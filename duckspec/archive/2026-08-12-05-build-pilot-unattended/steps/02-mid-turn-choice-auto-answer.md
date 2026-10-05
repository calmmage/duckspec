# Mid-turn choice auto-answer

While pilot is armed, auto-answer live `UserChoiceRequest` (recommended else first) via
the existing in-band select path.

## Prerequisites

- [x] @step unattended-auto-send-policy

## Tasks

- [x] 1. On `UserChoiceRequest` while armed, pick recommended/first and
         `answer_user_choice` + settle transcript without requiring UI focus

- [x] 2. @spec chat/build-pilot Mid-turn choice auto-answer: Armed pilot auto-selects first chip when none marked recommended

- [x] 3. @spec chat/build-pilot Mid-turn choice auto-answer: Armed pilot auto-selects recommended-marked chip over first
