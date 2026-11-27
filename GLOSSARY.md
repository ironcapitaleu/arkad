# Glossary

This file defines the business terms of this repository. A definition says what a thing is. It
does not say how the code stores, computes, or names it. Those details belong in the design
documents.

## How to Read an Entry

Each entry has a definition, and can add these lines:

- **Example:** one real case of the term.
- **Synonyms:** other words that mean the same thing. Use the entry name in new text.
- **Not to confuse with:** a term that a reader can take for this one.
- **Related:** other entries of this file.

The sections are contexts. A term means the same thing everywhere inside its section. If one word
means two things in two sections, each section has its own entry, with a qualifier in the name.

## How to Change This File

- If a document gives a term a new meaning, or adds a term that a reader can misread, change this
  file in the same PR.
- If a term leaves use, delete its entry.
- If this file passes 100 entries, the team splits it into one `GLOSSARY.md` per crate.

## SEC Filings

### Accession Number

The ID that the SEC gives to one filing.

- **Example:** `0000320193-26-000001`.
- **Synonyms:** `adsh`, in the Notes data set.
- **Related:** Filing.

### Amendment

A filing that corrects or adds to an earlier filing of the same entity. Its form ends in `/A`.

- **Example:** a 10-K/A that restates the revenue of a 10-K.
- **Related:** Filing, Form.

### Company

A business that the product shows to its users. One company can have several entities over time,
for example after a merger that creates a new parent.

- **Not to confuse with:** Entity.
- **Related:** Entity.

### Concept

The name of what a fact measures, as a taxonomy defines it.

- **Example:** `us-gaap:Revenues`.
- **Not to confuse with:** Standard Concept.
- **Related:** Fact, Standard Concept.

### Entity

The legal body that files with a regulator, as the regulator identifies it. The SEC identifies an
entity by its CIK.

- **Example:** CIK `0000320193`, Apple Inc.
- **Not to confuse with:** Company.
- **Related:** Company, Filing.

### Fact

One value that a filing reports for one concept, one period, one unit of measure, and one set of
dimensions.

- **Example:** the revenue of one entity for one quarter, in USD.
- **Related:** Concept, Fact Value, Filing, Unit of Measure.

### Fact Value

The number or the text that a fact reports.

- **Not to confuse with:** State Value.
- **Related:** Fact.

### Filing

A report that one entity submits to a regulator at one time.

- **Example:** the quarterly report of one entity for one quarter.
- **Related:** Accession Number, Amendment, Entity, Form.

### Form

The type of a filing. It says what the filing reports.

- **Example:** 10-K, the annual report of a US entity. 10-Q, its quarterly report. 20-F and 40-F,
  the annual reports of foreign entities.
- **Related:** Amendment, Filing.

### Instance

The XBRL document of one filing. It holds every fact that the filing reports.

- **Not to confuse with:** Notes File.
- **Related:** Filing, Source.

### Notes File

One file of the SEC Financial Statement and Notes data set. It holds the facts of every filing of
one month or one quarter, as tables.

- **Example:** the file of the first quarter of 2024.
- **Not to confuse with:** Instance.
- **Related:** Source, Split.

### Unit of Measure

What the number of a fact counts.

- **Example:** USD, shares, USD per share.
- **Not to confuse with:** Write Unit.
- **Related:** Fact.

## Storage Layers

### Bronze

The layer that keeps the files of the SEC as they arrived, unchanged.

- **Synonyms:** raw store.
- **Related:** Raw Document, Silver.

### Gold

The layer that the screener and the API read: the companies and their facts, ready to serve.

- **Related:** Silver.

### Raw Document

One file in bronze.

- **Example:** one Notes file, one instance.
- **Related:** Bronze, Source.

### Silver

The layer between bronze and gold. It holds the facts of every filing as filed, the standardized
facts, the findings, the company attributes as filed, and reference data such as taxonomy
releases, in tables that a query can read.

- **Related:** Bronze, Gold, Write Unit.

## Silver Data

### As-Is Fact

A fact as the filing reports it, with no choice among copies and no mapping to a standard concept.

- **Not to confuse with:** Standardized Fact.
- **Related:** Candidate, Fact, Value Key.

### Candidate

The value of one standard concept that one filing gives, before the filings of an entity are
compared with each other.

- **Related:** As-Is Fact, Standardized Fact.

### Copy Rank

The number that tells apart two facts of one filing that agree in every other part of their value
key. The first copy has rank 0.

- **Example:** a filing reports one revenue twice, once rounded to millions. The two copies get
  ranks 0 and 1.
- **Related:** Value Key.

### Current Unit

Of the write units of one unit key, the one that silver wrote last. A normal read takes only current units. The
results of the series step are the exception: a read takes the last full run that finished, and
the changes of each finished run after it.

- **Related:** Replay, Unit Key, Write Unit.

### Dataset

One table of silver, such as the as-is facts, the filings, or the standardized facts.

- **Not to confuse with:** Notes File.
- **Related:** Silver, Unit Key.

### Dropped Row

A row of a raw document that the transform leaves out of silver on purpose, with a reason.

- **Example:** a row of a form that the pipeline does not select.
- **Related:** Rejected Filing.

### Entity Bucket

One of a fixed number of groups of entities. The series step works on one set of buckets at a time.

- **Related:** Entity.

### Filing Day

The write unit of one filed date: every selected filing of that date, read from its instance.

- **Example:** the filings of 2026-11-06.
- **Related:** Write Unit.

### Finding

A problem or a remark that a step reports about the data without failing.

- **Example:** a gap in the quarters of an entity. Two values of one concept that disagree in one
  filing.
- **Related:** Candidate, Standardized Fact.

### Replay

A new run of the transform on the raw documents that bronze already holds. It writes new write
units next to the old ones, and deletes nothing.

- **Example:** a run after a fix of a parser.
- **Related:** Current Unit, Write Unit.

### Source

The kind of raw document that a fact came from: a Notes file or an instance. One filing can arrive
from both.

- **Related:** Instance, Notes File.

### Standard Concept

A concept of our own model. Several concepts of a taxonomy can map to one standard concept.

- **Example:** `Revenue`, which `us-gaap:Revenues` and `us-gaap:SalesRevenueNet` map to.
- **Not to confuse with:** Concept.
- **Related:** Candidate, Standardized Fact.

### Standardized Fact

The value of one standard concept for one entity and one period, after the filings of the entity
are compared and one value wins.

- **Not to confuse with:** As-Is Fact.
- **Related:** Candidate, Standard Concept.

### Unit Key

The dataset, the source, and the kind of a write unit, together with its name. The name alone
repeats across datasets.

- **Example:** the Notes file `2024q1` of the as-is facts.
- **Related:** Current Unit, Write Unit.

### Value Key

The parts that name one as-is fact within one filing and one source: the concept, the period,
the unit of measure, the dimensions, and the copy rank.

- **Related:** As-Is Fact, Copy Rank.

### Write Unit

The set of rows that one write adds to silver, and that one replay writes again.

- **Example:** every filing of one Notes file. Every filing of one filing day.
- **Synonyms:** unit, in the silver and transform documents.
- **Not to confuse with:** Unit of Measure.
- **Related:** Current Unit, Filing Day, Unit Key.

## Transform

### Filing Step

The work of the transform on one whole filing: read its facts, standardize them, and build its
rows for silver.

- **Not to confuse with:** Series Step.
- **Related:** Rejected Filing, Whole Filing.

### Rejected Filing

A filing that the filing step cannot process as a whole, for example because its header is broken.
The write unit goes on without it. Too many rejected filings fail the write unit.

- **Related:** Dropped Row, Filing Step.

### Series Step

The work of the transform that compares the candidates of every filing of one entity, and picks
its standardized facts.

- **Not to confuse with:** Filing Step.
- **Related:** Candidate, Entity Bucket, Standardized Fact.

### Split

How the transform cuts a Notes file into whole filings: in one pass when the file is sorted by
filing, or in several passes over groups of filings when it is not.

- **Related:** Notes File, Whole Filing.

### State Value

The data that one state of a state machine takes, or hands to the next state. In the transform, one
state value holds one whole filing.

- **Example:** "one value per filing": each state of the filing step takes and returns the data of
  one whole filing.
- **Not to confuse with:** Fact Value.
- **Related:** Whole Filing.

### Whole Filing

Every fact and header of one filing, gathered in one place before any step reads it.

- **Related:** Filing Step, Split.
