# Raw user inputs

Verbatim user messages for change `prose-option-next-actions`. Not a summary.

## Explore review option selection UI

/ds-explore

For review mode option selection
"""
**Options**

| | resolution | trade-off |
| --- | --- | --- |
| **A** | Stamp `session_harness` on id update; only pass `session_id` when harness matches selected model | Matches duckboard; small change |
| **B** | Clear `agent_session_id` whenever the selected harness changes | Simpler; may drop valid same-harness resumes if model string changes carelessly |
| **C** | Leave as-is (single-harness usage assumed) | Zero work; footgun remains |

**Recommendation:** **A** — field already exists on the shared session type; wiring is thin and pairs with finding 1’s send-path work.

---

How do you want to resolve **finding 2**?

- **A** — harness-scoped resume (recommended)  
- **B** — clear id on harness change only  
- **C** — accept / document
"""
Support an interface similar to next-action: show options in editor, tab to circulate between them

1 - Extract from prose if detected usual a-b-c- pattern with regexp
2 - all: letter + label + full text (clipped to fit into window on single line)
3 - not sure what chips mean.. I'm ok with either if it works, but i guess 
4 - any multi-op should be nice to support this.
5 - i don't understand the question

/auto

/ds-explore

create change prose-option-next-actions

/ds-propose

confirm proposal

/ds-design

> Lean

Accept all suggested recommendations, proceed

Accept all suggested recommendations, proceed

Fine

FOR FUCKS SAKE ACCEPT ALL RECOMMENDED SOLUTIONS AND GO ON BUILD ALREADY!!!

Go on

/ds-ar

/ds-review

A

B

By the way, why does this not work?

[clip-2026-08-12-18-39-00.png](attach:18cb18fa4b2ab3d0)

Should the 'next' section be absolutely last to be correctly processed? Let's relax that rule and fix that.

I don't understand. 

What exactly is uncovered by tests?
Would it be nice to cover it? How important of a case that is?

If yes - cover,
if no - skip

Go on

/ds-a

/ds-archive

confirm archive
