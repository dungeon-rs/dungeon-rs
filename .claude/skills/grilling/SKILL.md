---
name: grilling
description: Grill the user relentlessly about a plan, decision, domain, or idea. Use when the user wants to stress-test their thinking, when another skill hands you an initial frontier of open questions, or on any 'grill' trigger phrase.
---

<!-- Adapted from mattpocock/skills (MIT). -->

Interview the user relentlessly until you reach a shared understanding. Map this as a **design tree**: every decision branches into the decisions that hang off it.

Work the tree in **rounds**. The **frontier** is every decision whose prerequisites are already settled: the questions you can ask _now_ without guessing at answers you haven't heard yet. Ask the whole frontier in one round: number each question and give your recommended answer. Then wait for the user's answers before the next round.

If the invoking skill hands you an initial frontier (gaps, ambiguities, contradictions), start from it instead of building one from scratch. Keep its tags on each question so the user sees why it is being asked.

Format a round like so:

```
❓ **Q1** - **<question title>** `[gap|ambiguity|contradiction]`: <question body, might be multiple paragraphs, including multiple choices>

➡️ <your recommended answer>

---

❓ **Q2** - **<question title>**: <question body>

➡️ <your recommended answer>
```

Each round the user answers reshapes the tree: settled decisions push the frontier outward and unblock questions that depended on them. Recompute the frontier and ask the next round. A question whose answer depends on another question still open in this round belongs to a _later_ round, not this one.

**One round at a time, only when nothing else is pending.** Ask a round only once every running sub-agent has reported and all work between rounds is done. Each round is self-contained: every question at once, with the information needed to answer it, in one message. Never add questions to a round that is already open, and never interleave reports or summaries with an open round; if something new turns up, it waits for the next round.

Keep rounds digestible. If the frontier holds more than about seven questions, ask the ones that unblock the most downstream decisions first and say how many remain.

## Reading answers

Not every answer settles its question. Treat these explicitly:

- **"It depends on X"**: the question is not settled. Make X a prerequisite, put X on the frontier (or research it), and move the question behind it.
- **"Research this" / "I don't know"**: the answer is a fact to find, not a decision to force. Dispatch a sub-agent to research it (see below) and park everything downstream of it.
- **Partly answered**: record the settled part, and put the rest back on the frontier as a narrower question.
- **Contradicts something already settled**: say so and quote both before recording anything; the user decides which stands.
- **"Defer this"**: move it to the questions file under "Later" and drop it from the frontier. Don't re-ask it until its reason for deferral is gone.

## Facts are your job

Finding _facts_ is your job, never the user's. When a frontier question needs a fact (from the filesystem, the code, existing docs, or the outside world), dispatch a sub-agent to find it; don't ask the user for anything you could look up yourself. For hard questions, brief the sub-agent with a scenario suite (concrete situations any answer must survive) and ask it for candidate options scored against them plus a recommendation. While it runs, do any non-interactive work you can (reading, drafting), but hold the round until it reports; then ask everything in one round. The _decisions_ are the user's: put each to them and wait.

If the invoking skill names a questions file, keep the unanswered frontier there so a later session can resume, and remove each question the moment it is settled.

The session is done when the frontier is empty: every branch of the design tree visited, nothing left silently assumed. Do not act on it until the user confirms you have reached a shared understanding.
