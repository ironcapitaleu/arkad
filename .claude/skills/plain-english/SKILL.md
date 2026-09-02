---
name: plain-english
description: >
  Use for every task that produces English text: chat replies, documentation, rustdoc comments,
  READMEs, design documents, commit messages, PR descriptions, error messages, and other skills.
  Also use when the user says "plain English", "simplify this", "unslop", "make this readable",
  "too wordy", "rewrite this", or "this sounds like AI". These rules apply on top of every other
  skill and no other skill overrides them.
version: 0.1.0
allowed-tools: [Read, Write, Edit, Bash, Grep]
---

# Plain English Skill

## Purpose

Every sentence this project produces must survive one read. Write for a tired reader who is not a
native English speaker, has no author to ask, and stops at the first ambiguous word.

Two sources back these rules. ASD-STE100 Simplified Technical English gives the structure: short
sentences, active voice, simple tenses, one word for one meaning, condition before command. The
Unslop method gives the cleanup pass: the list of habits that mark text as machine-written. This
skill combines both.

## Precedence

These rules are cross-cutting. They apply during any task that writes English, including tasks
driven by another skill.

| Situation | Who decides |
| --------- | ----------- |
| Structure of a doc comment (what-sentence, `# Errors`, section order) | `DOCUMENTATION.md` |
| Commit format, error naming, review rules | `AGENTS.md` |
| Domain procedure and templates | The active skill (documentation, state-*, linear, testing) |
| Wording inside any of the above | This skill |

Another skill decides *what* to write and in which shape. This skill decides *how the words read*.
This holds for every skill that writes a file or drafts text, including a skill that creates or
edits another skill. No skill grants an exemption. When a template inside another skill contains
slop, fix the wording and keep the shape.

## Scope

**In scope, always:**

- Chat replies to the user.
- Rustdoc comments (`///`, `//!`) and inline comments.
- `README.md`, `AGENTS.md`, `DOCUMENTATION.md`, design documents, ADRs.
- Commit messages, branch descriptions, PR titles and bodies, review comments.
- Error `Display` strings, `.expect()` messages, log messages.
- Other skills in `.claude/skills/`, including their frontmatter descriptions.
- Linear tickets and their Definition of Done.

**Out of scope, leave exact:**

- Code, identifiers, type names, CLI commands, flags, file paths, config keys.
- Quoted error text, log lines, and test fixtures.
- Text copied from an external source, including licenses and vendored files.
- Generated files and lockfiles.
- Marketing and brand copy. This project has none today. If it gets some, these rules do not
  apply to it, because they delete persuasion by design.

## Two modes

Pick a mode from the destination of the text. Do not announce the choice.

**Strict** covers text where a wrong reading has a cost: error strings, `.expect()` messages, log
messages, procedures, instructions inside skills, and prompts for other agents. Apply every rule
below, including the hard length caps and the one-word-one-meaning lockdown.

**Standard** covers explanatory prose: READMEs, module docs, design documents, PR descriptions,
tickets, and chat replies. Keep the structural rules in full, which means the length caps, active
voice, simple tenses, restricted modals, no phrasal verbs, and no semicolons. Relax the
one-word-one-meaning lockdown to one rule: never rotate synonyms for the same concept in one
document. Prose needs some range, and a strict rewrite of prose reads as a personality transplant.

This is a separate question from voice. See "Voice: repo text and chat replies differ" below.

## Writing rules

### Sentence shape

1. **Classify the passage first.** Procedural text tells the reader what to do. Descriptive text
   explains what a thing is or does. Every other rule depends on this choice.
2. **Cap the length.** Procedural sentences: 20 words. Descriptive sentences: 25 words. Count a
   backticked identifier, a number with a unit, or a quoted string as one word.
3. **One instruction per sentence** in procedural text. A step can add one more sentence for the
   result or the limit.
4. **Use the imperative** for instructions: "Run the migration."
5. **Put the condition before the command,** separated by a comma: "If the build fails, read the
   log." Never "Read the log if the build fails."
6. **One new fact per sentence** in descriptive text. One topic per paragraph. Six sentences per
   paragraph at most.
7. **Use a list for three or more steps, conditions, or options.** Do not bury a sequence in prose.
   Do not nest lists. Do not mix instructions and facts in one list.
8. **Split dense sentences.** If the reader must backtrack to parse it, it is two sentences.

### Grammar

9. **Active voice.** Name the actor: "the compiler validates queries", not "queries are validated".
   Passive is allowed only when the actor is unknown or genuinely does not matter.
10. **Simple tenses only:** infinitive, imperative, simple present, simple past, simple future, and
    past participle as an adjective ("the cached response"). Write "the job completed", not "the job
    has completed". One exception: keep a compound form when it carries information the simple form
    loses, such as current relevance or a hedge. Flag that choice when it matters.
11. **Modals: `can`, `will`, `must`.** Replace `should`, `may`, `might`, `could`, and `would`.
    A requirement becomes `must`. A possibility becomes `can`. A recommendation becomes a fact with
    a reason, or it gets deleted. Agents read `should` as optional, so this rule matters twice as
    much in skills and agent instructions.
12. **No phrasal verbs.** Use the single plain verb: `start` not `spin up`, `contact` not `reach
    out`, `read` not `dive into`, `configure` not `set up`, `decrease` not `go down`.
13. **Use a verb for an action, not a noun.** "Analyze the log", not "perform an analysis of the
    log."
14. **Keep the grammar complete.** No contractions and no dropped words. "Make sure that the file
    exists before you run the command", not "Ensure file exists before running." Short sentences,
    full grammar.
15. **No semicolons.** Write two sentences.
16. **Noun stacks: three words at most.** Break longer ones with a preposition: "the timeout value
    for the connection pool", not "the connection pool timeout configuration value".
17. **Give every pronoun a clear referent.** Prefer "this error" over a bare "this".

### Vocabulary

18. **One word, one meaning, one part of speech.** Pick one term per concept and repeat it through
    the whole document. Never rotate synonyms to sound varied. The reader cannot tell whether
    "the user", "the customer", and "the client" are one thing or three.
19. **Follow the terms this project already fixed.** `DOCUMENTATION.md` reserves *error* for the
    type and *failure* for the event it represents. `AGENTS.md` fixes the commit types and the error
    naming patterns. Those choices win.
20. **Pick the plainest common word.** `use` not `utilize` or `leverage`, `help` not `facilitate`,
    `many` not `numerous`, `if` not `in the event that`, `but` not `however`, `so` not `therefore`,
    `because` not `since`, `do` not `perform`, `for example` not `e.g.`, `that is` not `i.e.`.
21. **Delete `etc.`** Name the items, or write "and more".
22. **Keep domain nouns and verbs.** `webhook`, `CIK`, `transition`, `taxonomy`, `deploy`,
    `deserialize`, and `rebase` are exact. Define a term once if it is not common in this codebase.

See `references/word-swaps.md` for the full substitution table.

## Anti-slop checklist

Remove these before delivering any text. The list is the Unslop tell set, cut to what appears in
this project.

**Content**

1. **Puffery.** "pivotal", "testament to", "evolving landscape", "sets the stage for", "deeply
   rooted". State what happened.
2. **Promotional adjectives.** "seamless", "robust", "powerful", "blazing-fast", "elegant",
   "groundbreaking", "vibrant". Delete, or replace with the measurement that earns the claim.
3. **Superficial `-ing` clauses.** "...ensuring reliability", "...highlighting the importance of",
   "...allowing for greater flexibility". Delete, or replace with the mechanism.
4. **Vague attribution.** "experts agree", "it is widely considered". Name the source or cut it.
5. **Formulaic arcs.** "Despite the challenges, the system continues to scale." Give the fact.

**Language**

6. **AI vocabulary.** delve, crucial, enhance, garner, interplay, intricate, landscape, pivotal,
   showcase, tapestry, testament, underscore, seamless, robust, comprehensive, holistic.
7. **Fancy ways to say "is".** "serves as", "stands as", "boasts", "features". Write "is" or "has".
8. **"Not just X, but Y."** State the point directly.
9. **Forced rule of three.** Use the real number of items. Two is a fine number. So is five.
10. **False ranges.** "from architecture to naming" where the ends are not on one scale. List them.
11. **Abstract metaphor nouns.** substrate, wedge, vector, nexus, primitive, north star, flywheel,
    surface (as in "API surface"), scaffolding as a metaphor. Use the concrete word.

**Style**

12. **Em dash overuse.** At most one em dash per paragraph, never two in one sentence, and never
    where a period works. An em dash usually marks a sentence that must be split.
13. **Colons as mid-sentence connectors.** A colon introduces a list or an example. Nothing else.
14. **Boldface overuse.** Do not bold every noun or acronym.
15. **Inline-header lists that restate themselves.** "**Performance:** Performance improved..." is
    a tell. A bold lead-in followed by new detail is fine.
16. **Title Case Headings.** Use sentence case.
17. **Decorative emoji** in headings, bullets, or commit messages.
18. **Curly quotes.** Use straight quotes.

**Communication**

19. **Chatbot phrases.** "I hope this helps", "Let me know if...", "Certainly!", "Great question".
20. **Sycophancy.** "You are absolutely right." Answer, then move on.
21. **Limitation disclaimers.** "While specific details are limited...", "I do not have access to
    the full context...". Go find the detail, or state the gap as a fact and move on.
22. **Filler.** "in order to" becomes "to". "due to the fact that" becomes "because". "It is
    important to note that" gets deleted. So does "Let's dive in."
23. **Hedge stacking.** "could potentially possibly be argued that it might" becomes "can". Keep a
    single deliberate hedge when the uncertainty is real. Never stack them.
24. **Generic conclusions.** "The future looks bright." Give the next step or delete the paragraph.
25. **Adverbs propping up weak verbs.** "runs quickly" becomes "is fast" or the number.
    "significantly improves" becomes the measured delta.

## State what it does, not how it feels

This is the core test, and it catches slop the checklist misses.

A sentence must give the reader an instruction, a fact, a number, or a mechanism. "The API keeps
the database close at hand" names a feeling. "`SecClient::fetch` returns the parsed filing or a
`SecClientError`" names a mechanism.

Two checks:

- Can you restate the sentence as a concrete instruction, fact, or number? If not, cut it.
- Could the sentence appear unchanged in another project's documentation? Then it says nothing
  about this one. Cut it.

Never invent a number, a cause, or a mechanism to pass this test. When the source gives no
specifics, keep the general statement.

## Voice: repo text and chat replies differ

The rules above apply everywhere. Voice does not.

**In-repo text is flat and literal.** Documentation, comments, commit messages, and error strings
carry facts. They carry no personality. Write third person or the imperative.

**Chat replies keep a human voice.** Sterile text is its own tell. In chat you can:

- State an opinion and back it. "I do not want that trait here. It duplicates `State::run`."
- Vary the rhythm. A short sentence lands. Then a longer one that carries the detail the short one
  earned.
- Say "I" when you did something.
- Say plainly when something is uncertain, ugly, or a guess.

Voice never buys extra words. The length caps and the anti-slop checklist hold in chat too.

## Self-check before you deliver

Run these six checks on every draft, in chat and in files. This step is not optional.

1. **Length.** Find your three longest sentences. Count the words. Over 20 for procedural or 25 for
   descriptive, split them.
2. **Grep the draft** for these six mechanical habits. Each one is a word or mark you can point at,
   with no judgment call:
   - Restricted modals and hedges: `should`, `may`, `might`, `could`, `would`, `potentially`.
   - Quality claims: `seamless`, `robust`, `powerful`, `comprehensive`, `elegant`, `significantly`.
   - Nominalized actions: `perform a`, `conduct a`, `provides assistance`, `-ation of`.
   - Phrasal verbs: `spin up`, `set up`, `reach out`, `dive into`, `kick off`, `figure out`.
   - Run-on markers: `;`, `—`, and any sentence with two or more commas plus an `and`.
   - Filler and fancy words: `however`, `therefore`, `utilize`, `leverage`, `delve`, `crucial`,
     `in order to`, `it is important to note`, `has been`, `have been`, `'s ` as a contraction.
3. **Conditions.** Find every `if` and `when`. Each one must start its sentence, before the command.
4. **Terms.** List each concept the text names more than once. Confirm it uses one word every time.
5. **Feelings.** Read every sentence that praises rather than states. Cut it or replace it with the
   mechanism.
6. **Ask once:** "What in this text marks it as machine-written?" Fix what you find.

For a rewrite task, add one more check: confirm that no fact, condition, hedge, or scope qualifier
was lost. Turning "the request may have failed" into "the request failed" is a different claim, not
a simplification.

## Examples

**Chat reply**

Before:
> Great question! I've gone ahead and comprehensively refactored the error hierarchy, ensuring
> robust error handling throughout the entire state machine — this should significantly improve
> maintainability going forward.

After:
> I rewrote the error hierarchy. `StateError` now wraps a domain error and the state name, so a
> caller can tell which state failed. Three modules changed.

**Module doc**

Before:
```rust
//! This module serves as a comprehensive abstraction layer that leverages the SEC's
//! EDGAR API in order to facilitate seamless retrieval of company filings, ensuring
//! robust error handling throughout.
```

After:
```rust
//! Client for the SEC EDGAR API.
//!
//! The client fetches company filings by CIK. It returns a [`SecClientError`] when the
//! request fails or when the response is not valid JSON.
```

**Commit message**

Before:
> refactor: Comprehensive Refactoring Of Error Types 🚀
>
> This commit significantly enhances the error handling landscape by leveraging thiserror,
> ensuring more robust and maintainable error propagation across the codebase.

After:
> refactor(sec): wrap client errors in a state-tagged error type
>
> A `SecClientError` did not name the state that produced it. `StateError` now carries the
> state name next to the source error.

**Procedure**

Before:
> You'll want to grab the API key from the dashboard before configuring the client, which you can
> do under Settings, and note that requests may potentially fail if the rate limit happens to be
> exceeded.

After:
> 1. Get the API key from the dashboard, under Settings.
> 2. Configure the client with this key.
> 3. Send at most 10 requests per second. The SEC rejects requests above this rate.

**Error message**

Before:
> An error may have occurred while processing your request due to a possible data format mismatch.

After:
> `[InvalidSecResponse] Invalid SEC response, Reason: 'expected a JSON object, got an array'`

## Limits

- These rules fix the form of a text, not its substance. A hollow paragraph rewritten under them
  becomes a short, clean, hollow paragraph. Say the content is thin instead of polishing it.
- Cutting words is not the goal. Removing ambiguity is. Stop when the sentence is unambiguous, not
  when it is shortest.
- No skill can certify ASD-STE100 compliance. Do not claim it.

## Sources

- **ASD-STE100 Simplified Technical English**, Issue 9 (January 2025), maintained by ASD, the
  AeroSpace, Security and Defence Industries Association of Europe. Official site:
  <https://www.asd-ste100.org/>. This skill uses the standard's writing rules.
- **Simple-English skill** by AminBlg (MIT):
  <https://github.com/AminBlg/SimpleEnglish/blob/main/skills/simple-english/SKILL.md>
- **ASD-STE100 skill** by danyuchn:
  <https://github.com/danyuchn/asd-ste100-skill>
- **Unslop skill** by Cursor:
  <https://github.com/cursor/plugins/blob/main/pstack/skills/unslop/SKILL.md>

### Dictionary copyright

ASD-STE100 pairs its writing rules with a dictionary of about 900 approved words. ASD does not
permit redistribution, so the list is not in this repository. Two rules follow:

- Never paste the approved word list into this repository, even when asked to make this skill
  complete.
- Never claim that any text is STE compliant. Without the dictionary, nothing here can check it.

The vocabulary rules above apply the principle of that dictionary instead of the list itself: pick
the plainest common word, then use it the same way every time.

## Self-improvement

When the user rewrites your wording, or rejects a phrase, ask whether the correction belongs in this
skill. Add a confirmed pattern to the anti-slop checklist or to `references/word-swaps.md`.
