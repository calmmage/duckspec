# Raw user inputs

Verbatim user messages for change `build-pilot`. Not a summary.

## Explore auto-advancing build workflows

/ds-explore

Add support for automated 'dynamic' workflows - auto-moving forward the item along the scenario.
e.g. once an agent finishes answering in chat - send the next response automatically

Option 1: build-auto. explore, design, spec, apply, steps, review, spec again until no review comments left

unless there's no 'next' section and some questions for the user they need to answer / critical issues (how to check that? by keywords?)

Option 2: build-fast build - just do each step once, sending 'confirm' and next step every time

Add new (system? or 'hybrid') commands '/ds-auto and /ds-fast'

Why system:
the command need to be recognized by the duckboard itself to start sending next messages to the agent automatically

Maybe then have it as just '/build-auto' and '/build-fast' instead of ds- commands?

1 - ok, build-*
2 - i guess we should be able to handle those commands at any stage before 'archived'..
Usual esc-esc should stop, yes
3 - yes, send the command to kick off the conversation
4 - yeah, maybe join design and spec in one message or something
5 - no, only stop on issues/questions detected by the response text
6 - yes

e.g. if i type /build-auto (description of idea) we send to agent ds-explore (description of an idea) instead

Do everything as recommended, ask everythign that's left

1 - fully off until re-launch
2 - i don't care. Show 'auto' mode plaque above input window, and then print in messages automatically, as if mine. Therefore show ds-explore.

3 - yes, live on change too, proceed by the same logic as i described

4 - nah, don't worry about restarts. I mean pause when app shuts down / something else breaks. - for now.
Add a note to consider having it reactivate automatically on error / crash / restart / whatever.
Have this as a setting, actually, default = off
5 - i guess pause, i don't know yet those commands
6 - yes

confirm

/ds-propose

/ds-propose

confirm

/ds-design

confirm

/ds-spec

confirm

confirm

confirm

/ds-step

confirm steps

/ds-apply

/ds-apply

/ds-apply

/ds-apply

/ds-apply

/ds-apply

/ds-apply

/ds-review

A

A

B

confirm review

/ds-spec

confirm map

confirm chat/build-pilot

/ds-step

confirm steps

/ds-apply

/ds-apply

/ds-review

A

confirm review

/ds-step

confirm steps

/ds-apply

/ds-review

clean archive-ready

confirm review

/ds-archive

## 2026-08-01 06:14 build-pilot

confirm archive
