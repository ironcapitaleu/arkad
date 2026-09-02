---
source: our own list, written for this repo
last-verified: 2026-09-01
update-frequency: on-demand, as new slop patterns are confirmed
---

# Word swaps

Lookup table for the `plain-english` skill. Read it when you rewrite existing text.

If a word carries no fact, delete it instead of replacing it.

These swaps are our own. They are not the ASD-STE100 approved word list, which stays out of this
repository. See "Dictionary copyright" in `../SKILL.md`.

## Connectives and filler

| Written | Write instead |
| ------- | ------------- |
| however | but |
| therefore, thus | so |
| since (meaning "because") | because |
| in order to | to |
| due to the fact that | because |
| in the event that | if |
| at this point in time, now | delete |
| it is important to note that | delete |
| it is worth mentioning | delete |
| as we can see | delete |
| let's dive in | delete |
| e.g. | for example |
| i.e. | that is |
| etc. | name the items, or "and more" |
| any (in "if you have any questions") | delete |

## Verbs

| Written | Write instead |
| ------- | ------------- |
| utilize, leverage | use |
| facilitate | help |
| perform (an action) | do, or the verb itself |
| conduct an analysis of | analyze |
| provide assistance to | help |
| enable, allow for | let, or name the mechanism |
| ensure, guarantee | make sure that |
| showcase, highlight | show |
| underscore | show, or delete |
| delve into | read, study |
| garner | get |
| enhance | improve, or the measured delta |

## Phrasal verbs

| Written | Write instead |
| ------- | ------------- |
| spin up | start |
| set up | configure, install |
| reach out | contact |
| dive into | read |
| kick off | start |
| go down | decrease, stop |
| take off | remove |
| figure out | find, decide |
| come up with | design, write |
| carry out | do |

## Adjectives that claim quality

Delete these, or replace them with the number or mechanism that earns the claim.

seamless, robust, powerful, comprehensive, holistic, cutting-edge, blazing-fast, effortless,
elegant, groundbreaking, vibrant, stunning, world-class, best-in-class, production-grade, battle-tested

## Abstract nouns

| Written | Write instead |
| ------- | ------------- |
| substrate | base |
| wedge (verb) | add |
| vector | way, method |
| nexus, locus | the thing itself |
| primitive (noun) | type, building block |
| surface (as in "API surface") | the API, the public items |
| scaffolding (metaphor) | the generated files |
| landscape, tapestry, ecosystem (abstract) | delete, or name the thing |
| north star, flywheel, endgame | the goal, the last phase |
| gold-plating | more than the job needs |
| paradigm, modality | the approach |
| testament to | evidence of, or delete |

## Modals

| Written | Write instead |
| ------- | ------------- |
| should (requirement) | must |
| should (recommendation) | state it as a fact with a reason, or delete |
| should a failure occur | if a failure occurs |
| may, might, could (possibility) | can |
| may (permission) | can |
| would (hypothetical) | can, or restructure as "If X, then Y" |
| shall | must |

## Terms this project has already fixed

For domain terms and naming conventions, follow the authoritative documents. Do not copy their
words here, because a copy goes stale when the source changes:

- `DOCUMENTATION.md` — domain vocabulary, such as `error` for the Rust type that carries a failure
  and `failure` for the event that went wrong.
- `AGENTS.md` — naming rules, such as the commit type prefixes, `Failed[Action]` for a failed
  action error, and `Invalid[Noun]` / `Missing[Noun]` / `Unexpected[Noun]` for a bad-state error.

For terms these documents have not fixed yet, pick one word per concept, then use it in the whole
document. Common pairs that need a choice: config or settings, run or execute, check or validate,
delete or remove, show or display. If a pick should hold across the project, record it in the
suitable authoritative document (`DOCUMENTATION.md` for domain terms, `AGENTS.md` for naming
conventions) so it becomes the source of truth.
