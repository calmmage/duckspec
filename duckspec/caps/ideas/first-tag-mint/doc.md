# First-tag idea mint

Create a linked idea for a free exploration or unlinked change so marks and tags have a
home — via first tag, or via mark-cycle mint when the row is still unlinked.

## When mint runs

```
| Gesture | Effect |
| --- | --- |
| First tag on free exploration / unlinked change | Create idea + set that tag |
| Mark cycle on free exploration / unlinked change | Create idea (no tags) + set star |
| Chat message | Does **not** create an idea |
```

```
| Target | Idea state | Title |
| --- | --- | --- |
| Free exploration | Exploration | Exploration display name |
| Active change without idea | Change | Prettified folder name |
```

## Linking

```
tag or mark on free exploration ──► idea (exploration = id)
                                    exploration.idea_path ──► idea

tag or mark on unlinked change  ──► idea (change = name)
```

Linked explorations stay on the CHANGE list.
