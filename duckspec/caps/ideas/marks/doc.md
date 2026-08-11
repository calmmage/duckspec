# Idea marks

Exclusive boost marks on ideas — none, star, hot, or cool — with a pin time for stars,
inherited by linked CHANGE and exploration list rows.

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
```

Rows without a linked idea expose no mark. Cycling mark on such a row does nothing —
creating an idea for annotation is owned by first-tag linking, not by the mark cycle.
