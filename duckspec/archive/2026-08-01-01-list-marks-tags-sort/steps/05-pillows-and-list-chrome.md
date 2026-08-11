# Pillows and list chrome

Type/phase pillow projection with density rules; wire CHANGE and Ideas lists to sort,
marks, pillows, and header sort menus.

## Prerequisites

- [x] @step queue-sort-and-list-prefs
- [x] @step first-tag-mint
- [x] @step mark-inheritance

## Tasks

- [x] 1. Project type pillows from secondary tags and phase from `change_scope_facts`
         under prefs

- [x] 2. Implement overflow policy: hide pillows on steady row when title+pillows exceed
         width; show on hover

- [x] 3. Extend CHANGE list: include idea-linked explorations, apply `sort_queue`, mark
         control, pillows

- [x] 4. Extend Ideas list: apply `sort_queue` within sections, mark control, type/phase
         pillows

- [x] 5. Add sort menu on Change section header and Ideas section headers writing shared
         `ListConfig`

- [x] 6. @spec ideas/queue-list Type and phase pillows with density: Secondary tags render as type pillows; primary does not

- [x] 7. @spec ideas/queue-list Type and phase pillows with density: Change-linked row can show derived phase pillow when pref is on

- [x] 8. @spec ideas/queue-list Type and phase pillows with density: When title plus pillows overflow row width, pillows hide until row hover

- [x] 9. @spec ideas/queue-list Type and phase pillows with density: Hidden pillow prefs suppress those pillows even when space remains

- [x] 10. Manually verify sort menu chrome on Change and Ideas headers (covers
          `ideas/queue-list` List preferences and sort menu: Sort menu is available on
          Change and Ideas section headers)

## Outcomes

- Sort menu is present on Change and Ideas section headers (task 10 verified in code
  paths: `sort_menu_controls` / `ideas_sort_menu`); visual polish can wait for dogfood.
