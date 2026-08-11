# Raw user inputs

Verbatim user messages for change `list-marks-tags-sort`. Not a summary.

## Explore favs tags and sorting

/ds-explore Allow favs, tags and labels on changes / ideas

Implement re-sort logic / custom sort rules for the change / ideas list

1) allow emojis - star, fire ('hot'), ice ('cool') - at the beginning of the change / exploration name - support this in data model
2) allow tags / labels - visible as pillows.  
limit tag length, in general care about visual density
If tag + title go over line length - only show tags on hover.
Tags = topics, maybe type (feature / component / bugfix etc) 

Allow using favs as part of the sorting logic
(Namely, I imagine that I want 3 most recent favs boosted up in the queue - to the top) If more than 3 favs - only boost 3, keep the rest ordered with the rest of sorting logic rules.

Possible splits:
- keep primary for tree; secondary tags only as row pillows  
- separate `type:` from free tags  
Yes, do that. Keep primary for tree. type tags pillow. And we're also designing to show status as pillow (make configurable to show/hide each of these)

### 4. Sort rules (beyond top-3 favs)
i was thinknig that we should auto-sort by last message timestamp
Also, sort by status i guess
(have a 'sort' button for settings - near change bar 'Plus' and open menu to select options)

### 5. Interaction model
Do this:
- click star on hover (row affordance - star outline appears on title hover )  
- cycle emoji on the title on multiple clicks.

> Open: are star / hot / cool **exclusive** (one mark) or **combinable** (⭐+🔥)

Exclusive. Star or fire or ice

> Idea is the annotation surface; change row inherits from linked idea
This one, yes

> Is pillow-status something else?
Yeah, i meant Duckspec phase

B - what do you think? 
I am not sure. 

and yes i said we should have length limit - if over length - just hide and show only on hover, same as status 

> Free exploration (no idea)
Don't we have a place to store the fav for that one? How about auto-create an idea from first message?

1 - yes
2 - first tag, i guess
3 - reuse, i guess

confirm

/ds-propose

confirm

/ds-design

confirm

## Open question
- Should Ideas section headers get the same sort menu, or only CHANGE + shared prefs?
Yes, i guess.


- On re-cycle back to Star, always refresh `favored_at` (new pin time) — confirm vs preserve original star time.

It's ok to refresh the time
    
- Unlinked change first tag: idea title = raw folder name or prettified slug?
Prettified slug I guess

/ds-spec

confirm

confirm

Go on

go on

confirm

confirm

/ds-step

confirm

/ds-apply

/ds-apply

/ds-apply

/ds-apply

/ds-apply

/ds-apply

/ds-review

A or B. Definitely not C or D
Elaborate what is A and what is B, i am unclear

FOR FUCKS SAKE EXPLAIN PLAINLY!!!

- WHAT THE FUCK IS 'MINT'?

WHAT THE FUCK IS THE DIFFERENCE BETWEEN
> user adds a tag on a CHANGE row (exploration or unlinked change).
AND 
> user clicks the star/cycle control on a CHANGE row that has no idea.

Actually, i understand 'click on a star' (we're going to have it in the beginning of the change title, right?

I don't understand what 'adds a tag on a row' - adds a tag how? What is the UX of adding a tag?

In any case, build option B right now, and clarify A in text, maybe we'll build that too - if i understand it.

Why did we even need to create an idea again? Because that's the only thing shared between change and exploration?

/ds-review

/ds-archive

reject archive

confirm archive
