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

| Concept | Word to use | Source |
| ------- | ----------- | ------ |
| The Rust type that carries a failure | error | `DOCUMENTATION.md` |
| The event or action that went wrong | failure | `DOCUMENTATION.md` |
| The commit type prefixes | feat, fix, refactor, style, perf, test, doc, ci, cd, build, revert, chore | `AGENTS.md` |
| A failed action error name | `Failed[Action]`, never `[Action]Failed` | `AGENTS.md` |
| A bad-state error name | `Invalid[Noun]`, `Missing[Noun]`, `Unexpected[Noun]` | `AGENTS.md` |

Pick one word per concept for terms this project has not fixed yet, then use it in the whole
document. Common pairs that need a choice: config or settings, run or execute, check or validate,
delete or remove, show or display.
