# Idea marks

Exclusive boost marks on ideas — none, star, hot, or cool — with a pin time for stars,
inherited by linked CHANGE and exploration list rows.

Exclusive boost marks on ideas — none, star, hot, or cool — with a pin time for stars,
inherited by linked CHANGE and exploration list rows, and painted as one interactive
control (not a title prefix).

## Marks

An idea carries at most one boost mark:

```
none → star → hot → cool → none
```

```
| Mark | Role |
| --- | --- |
| none | Unmarked |
| star | Favorite; participates in pin-to-top ordering elsewhere |
| hot | Temperature signal only for this capability |
| cool | Temperature signal only for this capability |
```

Cycling always follows that ring. Marks live on the idea; they are not stored on change
folder names or exploration records.

## Pin time

Only *star* has a pin time. Entering star records it; leaving star clears it; entering
star again records a fresh time so later “most recently starred” ordering sees the latest
star gesture.

## Inheritance

```
idea.mark
    │
    ├── CHANGE row for linked change
    └── CHANGE row for linked exploration
    └── Ideas row for the idea
```

Rows without a linked idea start with mark *none*. Cycling the mark control on an unlinked
CHANGE or exploration row mints a linked idea and applies the next mark (mint-on-click).
Tagging remains a separate path for creating annotation homes.

## Row presentation

List rows show the mark only as a single interactive control after the type icon (or
equivalent), never as a glyph prefix on the title text:

```
| Mark | Control |
| --- | --- |
| none, not hovered | hidden |
| none, hovered | outline star (click cycles / may mint) |
| star / hot / cool | always shown; click cycles |

```

Title labels stay the plain display name or idea title.
