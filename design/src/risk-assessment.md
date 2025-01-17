# Risk Assessment

This page keeps track of risks that we might face during development so that we can mitigate or even eliminate them.

## Organizational Risks

### Feature Creep

[_Feature creep_](https://en.wikipedia.org/wiki/Feature_creep) refers to the continuos addition of features that goes way beyond the initially planned scope of the game.
We are quite vulnerable to this for multiple reasons:

- The game genre and theme invites for many complex features and mechanics that can be combined freely.
  It's very easy to come up with additional ideas that would fit well into the game.

- As an open source project, we are going to have many different contributors with different backgrounds.
  Everyone might have their own ideas that they want to integrate into the game.

To mitigate this risk, the following will be important:

- Clearly document the vision of the game in the design book.
  This will help to align all contributors on what we want to build.

- Organize the bigger development plans in GitHub projects and/or milestones.
  Closely align this plan with the vision laid out in the design book.

- Focus on the core game loop first and then iteratively plan the next set of features we want to add.
  Keeping the scopes small for each iteration will ensure that we don't end up starting too many projects at once.

- Focus on the player experience.
  Get a playable version ASAP and then improve based on the player feedback, while being mindful of the larger vision.

### Financial Instability

As an open source project it will be harder to generate funding for the work we cannot do ourselves.
As an indie studio, there is also the uncertainty how popular the game will be in the end.
If we have full time contributors, it's important that we can sustain their work in the long term.

A Patreon model or GitHub sponsors could help with this.

### Rip Offs

Since the game is open source, other individuals or companies could sell the game without any major contributors.
They might also re-bundle the game with different assets.
This could hamper our own ability to sell the game.

One possible mitigation would be the use of different licenses for some aspects of the game.
For example, we could use CC non-commercial or share alike licenses for assets and copyleft licenses for non-reusable code.
However, this could constrain code reuse for good faith individuals and would also require the consent of all contributors.

### "Piracy"

Because the game is open source, players could simply compile it from source for free instead of buying it.
While this is completely legal and technically piracy, it could decrease our sales.

If this is going to be an actual problem remains to be seen.
Non-technical players probably won't want to go through the hassle to compile from source
(even though it's quite easy in the current state).
Some players may also want to buy it on Steam simply to have all games in one place and for achievement tracking.

### Contributor Fluctuation

As an open source project, we can rely less on the availability of individual contributors.
Sometimes life happens and the activity of some people might drop significantly, maybe without prior notice.
Some contributors will end their work on the project entirely and move on to other projects.
We will also get new contributors that might need some mentoring to get productive.

If we have one or more full time contributors, this could help mitigate this risk.
Useful documentation can help to get new contributors up-to-speed more quickly.
Code reviews and reliable CI workflows can help to maintain quality and consistency in our code base.

### Skill Bias

It's very likely that we are going to have way more people comfortable with programming than other fields.
Especially art assets and animations might become a bottleneck for development.

This could be mitigated by contract work, but this could introduce a significant financial burden to the project.
Otherwise, we should try to attract more artistic contributors.

## Technical Risks

### Bevy is still Experimental

While Bevy is already being used in production and has an impressive development speed,
it's still in its early stages compared to other game engines.
Some features that we are going to need might still be missing or more difficult to implement.
There are way less assets and resources ready to grab for prototyping or accelerated development.

Fortunately, we have some contributors who are very experienced with Bevy.
For our game we are also less reliant on features like a visual editor,
because we can procedurally generate the world.

### Performance-Sensitive Game

Factory builders and automation games are usually computationally expensive.
There are often large maps, tons of machines/structures and potentially many units with an AI.

Hopefully Bevy can help us to use all resources efficiently.
