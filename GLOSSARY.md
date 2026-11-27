# Glossary

This file defines the business terms of this repository, for every crate. Each entry says what the
term means here, and links the document that owns it. The owning document holds the full rules. If
an entry and its owning document disagree, the owning document decides, and the entry needs a fix.

If this file grows too large to read, it splits into one `GLOSSARY.md` per crate.

Two words have two meanings each: [unit](#unit) and [value](#value). Their entries say which
meaning each document uses.

## Accession Number

The ID that the SEC gives to one [filing](#filing), for example `0000320193-26-000001`. The Notes
data set names it `adsh`. Silver holds it inside `filing_id`, as `sec:<accession number>`.
Owner: [`xbrl/DESIGN.md`](xbrl/DESIGN.md#filing).

## As-Is Fact

One reported fact of one [filing](#filing), as filed, with no dedup and no mapping to a standard
concept. Both parsers of `xbrl::adapter::edgar` give as-is facts, and silver stores them in
`as_is_facts`. Owner: [`xbrl/DESIGN.md`](xbrl/DESIGN.md#asisfact).

## Bronze

The raw store: the SEC files as extract stored them, unchanged, each with a JSON metadata file.
Owner: [`sec/design/pipeline/silver_layer.md`](sec/design/pipeline/silver_layer.md#layers).

## Candidate

The value of one standard concept from one [filing](#filing), after steps 1 and 2 of the dedup
order and before "newest filing wins". Standardize filing writes candidates to `candidates`, and
the series step reads them. Owner:
[`sec/design/pipeline/silver_layer.md`](sec/design/pipeline/silver_layer.md#candidates).

## Copy Rank

The number that tells apart two [as-is facts](#as-is-fact) of one filing with the same
[value key](#value-key) except the copy rank. Rank 0
is the first copy. The parser assigns it, so the [value key](#value-key) stays unique. Owner:
[`xbrl/DESIGN.md`](xbrl/DESIGN.md#value-key).

## Current Unit

For one [unit key](#unit-key), the [unit](#unit) with the latest `written_at` in `silver_files`.
A default read of silver takes only current units. Owner:
[`sec/design/pipeline/silver_layer.md`](sec/design/pipeline/silver_layer.md#replays).

## Digest Group

The rows that `content_digest` hashes together: the rows of one [filing](#filing), or, for rows
without a filing, the rows of one [entity](#entity) or one taxonomy release. Owner:
[`sec/design/pipeline/silver_layer.md`](sec/design/pipeline/silver_layer.md#silver_files).

## Dropped Row

A source row that the transform leaves out of silver on purpose. `dropped_rows` of `silver_files`
counts them per [unit](#unit) by reason, for example `form_not_selected` or `rejected_filing`.
Owner: [`sec/design/pipeline/silver_layer.md`](sec/design/pipeline/silver_layer.md#silver_files).

## Entity

The company that files, as its regulator names it: a scheme and an ID, for example the SEC and a
CIK. Load maps an entity to a company with `resolve`
([`sec/design/pipeline/transform_per_filing.md`](sec/design/pipeline/transform_per_filing.md#where-resolve-and-the-checks-sit)).
Owner: [`xbrl/DESIGN.md`](xbrl/DESIGN.md#filing).

## Entity Bucket

One of 64 groups of [entities](#entity), by a hash of the scheme and the ID. The series step runs
per set of buckets, and `candidates` sorts its rows by bucket. Owner:
[`sec/design/pipeline/silver_writer_and_jobs.md`](sec/design/pipeline/silver_writer_and_jobs.md#entity-buckets).

## Filing

One report that a company submits to its regulator, for example one 10-Q. A filing has one
[accession number](#accession-number), one [form](#form), and the facts of that report. Silver
holds one row per filing in each [unit](#unit) that holds it, in `filings`. A read keeps one, by
the rules of "Current Facts". Owner:
[`xbrl/DESIGN.md`](xbrl/DESIGN.md#filing).

## Filing Day

The [unit](#unit) kind `filing_day`: every selected [filing](#filing) of one filed date, read from
the stored [instances](#instance). Its unit key is the date, for example `2026-11-06`. Owner:
[`sec/design/pipeline/silver_layer.md`](sec/design/pipeline/silver_layer.md#the-write-unit).

## Filing Step

The transform work on one [filing](#filing): parse, standardize filing, and build the silver rows.
A state machine per filing runs it, with no I/O. Owner:
[`sec/design/pipeline/transform_per_filing.md`](sec/design/pipeline/transform_per_filing.md#the-filing-step).

## Finding

A fact that a step reports without failing, for example a `Conflict` or a `Gap`. Silver stores the
findings of the standardize steps in `findings`. Owner:
[`sec/design/pipeline/silver_layer.md`](sec/design/pipeline/silver_layer.md#findings).

## Form

The type of a [filing](#filing), for example 10-K or 10-Q. A form with `/A` is an amendment. Both
paths keep the forms of `SELECTED_FORMS`: 10-K, 10-Q, 20-F, 40-F, and their amendments. Owner:
[`sec/design/pipeline/extraction_overview.md`](sec/design/pipeline/extraction_overview.md#filings-we-select).

## Gold

The serving layer in Postgres: the company brain that the screener and the API read. Load fills it
from [silver](#silver). Owner:
[`sec/design/pipeline/silver_layer.md`](sec/design/pipeline/silver_layer.md#layers).

## Instance

The XBRL instance document of one [filing](#filing): an XML file with every fact of the filing.
The new-filing path stores the main instance of each filing. Owner:
[`sec/design/pipeline/extraction_overview.md`](sec/design/pipeline/extraction_overview.md#xbrl-instance-and-label-file).

## Notes File

One ZIP of the SEC Financial Statement and Notes data set. It holds the [filings](#filing) of one
month or one quarter, its Notes period, in tables such as `sub.tsv`, `num.tsv`, and `txt.tsv`.
Owner: [`sec/design/pipeline/ingestion_paths.md`](sec/design/pipeline/ingestion_paths.md#backfill-extract).

## Raw Document

One file of [bronze](#bronze), such as a [Notes file](#notes-file), an [instance](#instance), or a
`submissions` file. Each silver row names the raw document it came from. Owner:
[`sec/design/pipeline/ingestion_paths.md`](sec/design/pipeline/ingestion_paths.md#raw-store).

## Rejected Filing

A [filing](#filing) whose [filing step](#filing-step) returns an error, for example a `sub` row with
a broken fiscal year end. The filing leaves its [unit](#unit), and too many rejected filings fail
the unit. Owner:
[`sec/design/pipeline/transform_per_filing.md`](sec/design/pipeline/transform_per_filing.md#rejected-filings).

## Replay

A run of the transform on the stored [raw documents](#raw-document) again, for example after a
parser fix. A replay adds a new [unit](#unit) next to the old one, and overwrites no file. Owner:
[`sec/design/pipeline/silver_layer.md`](sec/design/pipeline/silver_layer.md#replays).

## Silver

The processed layer between [bronze](#bronze) and [gold](#gold): Parquet datasets of as-is facts,
filings, candidates, standardized facts, findings, and reference data, and the list of written
units, `silver_files`. Owner:
[`sec/design/pipeline/silver_layer.md`](sec/design/pipeline/silver_layer.md).

## Source

The kind of [raw document](#raw-document) a silver row came from: `notes` or `instance`. A
[filing](#filing) can reach silver from both sources, and a read takes the instance. Owner:
[`sec/design/pipeline/silver_layer.md`](sec/design/pipeline/silver_layer.md#current-facts).

## Split

How the Notes reader cuts a [Notes file](#notes-file) into whole [filings](#filing): `stream`, one
pass over sorted tables, or `buckets`, K passes over hash buckets of `adsh`. Owner:
[`sec/design/pipeline/transform_per_filing.md`](sec/design/pipeline/transform_per_filing.md#the-split).

## Standardized Fact

One value of one standard concept of our model, such as `Revenue`, for one [entity](#entity) and
period, after dedup across filings. The series step writes them to `standardized_facts`. Owner:
[`xbrl/DESIGN.md`](xbrl/DESIGN.md#standardizedfact).

## Unit

This word has two meanings:

1. **Write unit.** The set of silver rows that one write adds, and one [replay](#replay) writes
   again: one [Notes file](#notes-file), one [filing day](#filing-day), one part of a series run,
   or one taxonomy release.
   The silver and transform documents use "unit" in this meaning. Owner:
   [`sec/design/pipeline/silver_layer.md`](sec/design/pipeline/silver_layer.md#the-write-unit).
2. **Unit of measure.** The measure of a numeric fact, for example `USD` or `USD/shares`. Documents
   write "unit of measure" or the column `unit`. Owner:
   [`xbrl/DESIGN.md`](xbrl/DESIGN.md#asisfact).

## Unit Key

The dataset, the [source](#source), the unit kind, and the column `unit_key` of a write
[unit](#unit). The column holds a name such as the Notes period `2024q1` or the date `2026-11-06`.
Owner:
[`sec/design/pipeline/silver_layer.md`](sec/design/pipeline/silver_layer.md#the-write-unit).

## Value

This word has two meanings:

1. **Fact value.** The number or text that a fact reports, such as `value_numeric` in
   `as_is_facts`. The silver and `xbrl` documents use "value" in this meaning. Owner:
   [`xbrl/DESIGN.md`](xbrl/DESIGN.md#asisfact).
2. **Rust value.** One value of a Rust type that a state takes or outputs. The state machine
   documents use "value" in this meaning: one value per filing holds all the facts of that filing.
   Owner:
   [`sec/design/pipeline/transform_per_filing.md`](sec/design/pipeline/transform_per_filing.md#one-value-per-filing).

## Value Key

The fields that name one [as-is fact](#as-is-fact) within a [filing](#filing): the filing, the
concept, the taxonomy version, the period end, the quarters, the unit of measure, the dimensions,
and the [copy rank](#copy-rank). Owner: [`xbrl/DESIGN.md`](xbrl/DESIGN.md#value-key).
