# Raw user inputs

Verbatim user messages for change `ghost-fill-and-editor-nav`. Not a summary.

## Explore suggestion and editor hotkeys

/ds-explore 
[clip-2026-08-12-14-29-12.png](attach:18cb0b58a0147b10)

Let's add a hotkey 'tab' and '->' right arrow to fill in the grey suggestion (but not send)

I guess we will need to make Shift+Tab for rotating between options, instead of Tab (which is current rotation hotkey, right?)

Also, for the editor window, can we please make Cmd + -> / <- go to the end of the line, not to next word...

/ds-propose /ds-step

1. **Cycle direction** — Shift+Tab forward-only wrap, or keep reverse somehow?
i don't really need reverse, there's never more than 3 options..
  
2. **Word jump** — bind Option+← / → after Cmd becomes line, or leave word-nav only on Home/End-style keys?
Yeah

create change ghost-fill-and-editor-nav

/ds-propose

confirm proposal

/ds-design

Yes, go with A

Accept

Hmmm. 

I am a little unclear.

Let me tell in my own words:
Desired behavior
If the option 2 is currently selected by the Shift+Tab rotation - the Tab button spawns THAT text in the window

Fine, A is good

A

P

confirm design

/ds-spec

confirm map

confirm chat/default-prompts

confirm editor/line-word-nav

/ds-step

confirm steps

/ds-apply

/ds-apply

/ds-review

Hm, i don't get it:
> Gate landmarks only when `chat_input_focused` is false **and** content has no focused editor

Why are we gating that we're NOT in the chat input?

The main idea is exactly to allow cmd left/right arrow line navigation INSIDE ai chat input..

Lol, right now it does both..

So. When input is selected - only jump inside input.
When not , but inside chat - ok - jump answers

Go on - /ds-design, /ds-spec, /ds-step and build

/ds-review

confirm review

/ds-archive

confirm archive

commit
