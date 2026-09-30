# Golem DB Design — Change Register

Single place where the three reviews of `golem-db-design.md` (revision `3748e29`, tag
`design-review-2026-09-24`) converge. The reviews are evidence and are not edited; this register
is the working record.

**Evidence** (outside this repository, in `warburg-workspace/`):

| Review | Lens | Path |
|---|---|---|
| Rubric (`R#`) | Structure vs. the best database design documents | `review/golem-db-design-vs-design-doc-rubric.md` |
| Fable (`F-…`) | Seven reader personas, comprehension | `personas/fable/persona-review-golem-db-design.md` |
| Astra (`A-D##`) | Six reader personas, conformance and correctness, checks C1–C7 | `personas/astra/astra_review_golem-db-design.md` |
| Programme | Consolidation and plan | `review/golem-db-design-change-programme.md` |

## How rows move

- **Kinds.** K1 fix (answer already implied by the doc; PR). K2 spec decision (`[D]` issue,
  decision-log entry, then spec PR). K3 structure (one shape decision, then mechanical moves).
  K4 product (product owner decides; decision-log entry tagged `product`).
- **Statuses.** `proposed → triaged → accepted | rejected | deferred → done`. K2 rows pass through
  `waiting_on_additional_input` between the write-up in `decisions_for_architect.md` and the
  architect's choice. A row is `done` when
  its acceptance check is satisfied, not when text has been edited.
- **Priority.** P1 blocks freezing the affected mechanism as a contract; P2 blocks integration or
  adoption decisions; P3 navigation and onboarding.
- One spec PR touches one row (K1 rows may be grouped, at most five per PR, one hunk per row).
  Interface changes carry `[API]` in the PR title. Bytes are agreed by fixture in
  `conformance/vectors/`, not by prose. Nobody writes "settled" until Phase 5 of the programme.
- K2 rows are worked in two rounds. Round 1 writes the decision up in
  [decisions_for_architect.md](decisions_for_architect.md) — question, alternatives,
  recommendation, rationale, closing fixture — for the architect and any external reviewer to judge
  from that file alone. Round 2, after the decision is recorded there, lands the spec text.

## K1 — Fixes

PR groups: **G1** F01 F10 F17 S03 S02 · **G2** F02 F03 F04 F05 F06 · **G3** F07 F08 F18 F11 F14 ·
**G4** F09 F12 F13 F15 F16 · **G5** S04 S12 · **G6** F19–F28 (reconciliation of 2026-09-30, landed
with the v2 adoption; see the triage log).

| ID | Status | Pri | Grp | Change | Sources | Acceptance check | Link |
|---|---|---|---|---|---|---|---|
| F01 | done | P1 | G1 | Repoint the three API links to `golem-db-api.md`; one-line precedence statement (design vs. API) | A-D01 | Links resolve; kind-bit, branch ops, sort tie-break agree between the two files | design.md §1 |
| F02 | done | P1 | G2 | Figure 5: remove unary branch `MID1`; third leaf attaches to root slot 1; root `tree_mask` = `0x0001`; fix Figure 6 | A-D03 | Fixture built from the three paths yields the drawn masks | design.md §6 Figures 5, 6 |
| F03 | done | P1 | G2 | Shard start ordinal = `mark(S−1)`, not `mark(S)`; marks are absolute high-water values | A-D08 | Nonempty boundary-commit example; Astra C3 passes | design.md §11 The Model, The System Segment |
| F04 | done | P3 | G2 | §4 prose: record 0's key is `#params`, not `#alloc` | A-D14 | Text matches catalogue | design.md §4 Common properties |
| F05 | done | P3 | G2 | Example `recordID`s 42/43 → ≥ 64 everywhere (§3, §8, §12), or mark abstract | A-D14 | grep finds no user example below 64 | design.md §3, §6, §8, §12: 42–47 → 100–105 |
| F06 | done | P3 | G2 | §10 overlay table: only change-set tables keyed by `commitNr`; history tables by item | A-D14 | Text matches dictionary | design.md §10 In-Memory Overlay, not-mirrored table |
| F07 | done | P1 | G3 | §7 historical example Phase 2: time-travel `#key` like every other cell | F-A3 | Example correct for a record deleted after T | design.md §7 Historical Query Execution Example, Phase 2 |
| F08 | done | P1 | G3 | Figure 10: failed op applies nothing (call-level atomicity); frame rollback undoes earlier successful ops | F-A2, A-D05.2 | Figure agrees with property 7 | design.md §10 Figure 10, caption, point 3 |
| F09 | done | P2 | G4 | Commit step 8: `#alloc` value already in the state hashed at step 6; step 8 persists it | F-Heph3, A-D05 | Nine steps consistent with §4 | design.md §10 Committing a Branch, steps 4, 7, 8 |
| F10 | done | P2 | G1 | §4 catalogue ↔ §11: add `#rootIndex` (id 4), `#shardSpan`, `#immutableDataSegments` with codecs, or mark §11 additions as proposed | F-A6, A-D12 | One catalogue, no orphan additions | design.md §4 catalogue, `#params` table |
| F11 | done | P3 | G3 | §11 cross-reference "[golem-db-design.md §4]" → in-document anchor | F-Ariadne | Link resolves | design.md §11 What This Assumes of the Schema |
| F12 | done | P3 | G4 | Rename "Bitemporality" → "Point-in-time history"; one sentence: one time axis (commit), no valid time | R8, A-D14, F-Sam | Term gone | design.md §7 title and first paragraph; 13 references |
| F13 | done | P3 | G4 | "Kind and type fixed per record" → "per record, per write" (retyping allowed, per API) | A-Maya/Sam | Agrees with API retyping rule | design.md §3 Cell Kinds and Types; §5; §12 |
| F14 | done | P3 | G3 | Label `WHERE …` / `FIND RECORDS …` as illustrative pseudo-syntax | F-D14, A-D14 | Label present | design.md §5 Notation callout; §7 example fence |
| F15 | done | P2 | G4 | Narrow overstated claims: §11 index page pollution, compression prohibition, blob pre-image; §8 "impossible" → "under collision resistance"; §8 "one container below 65 536 records" → allocation extent | A-D13, A-D11 | Each claim carries its assumption | design.md §8 Domain Separation; §8 Single-item tries; §11 Why Cells Are the Wrong Shape (2); §11 Rejected Alternatives, blob (2) |
| F16 | done | P2 | G4 | ABCI row: verify against CometBFT 0.38; expected `seal ↔ FinalizeBlock`, `commit ↔ Commit` | F-A5 | Citation to spec section | design.md §10 Why the Split Exists, table + note; spec v0.38 abci++_methods |
| F17 | done | P3 | G1 | Move the two "validate against reth source" hedges here as tasks; leave a footnote | F-D15 | Rows exist below | T01, T02 |
| F18 | done | P1 | G3 | §7 retention note lists `IndexHistory` / `IndexChangeSet` / historical bitmap nodes as query dependencies | A-D07 | Note matches example | design.md §7 retention note |
| F19 | done | P3 | G6 | Glossary: segments no longer "Proposed" (P05 adopted §11) | Reconciliation R1 | Glossary agrees with P05 | design.md Glossary, *segment / shard / row / ordinal* |
| F20 | done | P3 | G6 | Status by Chapter: drop P02 and "P05 (rows marked proposed)" from §4; mark P06 "decided, text not yet applied" in §1 and §7 | Reconciliation R2 | Status lines agree with K4 | design.md Status by Chapter |
| F21 | done | P2 | G6 | Open Questions: add the missing D19 row | Reconciliation R3 | Every D row cited in design.md has an Open Questions row | design.md Open Questions |
| F22 | done | P2 | G6 | Open Questions: remove the P01–P07 row (all decided or done); note that P06 and P07 are not yet applied to the text | Reconciliation R4 | Open Questions lists only open items | design.md Open Questions, note below the table |
| F23 | done | P3 | G6 | Stop citing `golem-db-design-short.md` as current: removed from both reading routes; intro link kept with a "lags, do not rely on it" caveat | Reconciliation R5; T03 | No reading route cites the short document | design.md intro; Who Should Read This |
| F24 | done | P3 | G6 | Name the source of requirement IDs (SE-1, CS-5, …): `arkiv-source-of-truth/golem-db.md`, outside this repository | Reconciliation R6 | Every requirement ID has a named source | design.md intro |
| F25 | done | P2 | G6 | Restore the §13 fingerprint argument dropped by S12; heading "(tracked as D11)", D11 row points to it | Reconciliation R7 | No v1 reasoning lost; D11 links to the argument | design.md §13 Open Question on Paging; Open Questions D11 |
| F26 | done | P3 | G6 | New Figure 11: a transaction that keeps its fee through `OutOfBudget` (fee op, checkpoint, user ops, one `rollback()`, settlement from `spent`). Figure 10 unchanged | Reconciliation R8 (A3 sign-off) | Figure agrees with the fee note and with property 7 | design.md §10 Figure 11 |
| F27 | done | P3 | G6 | §4: record 2 is touched at every commit, record 1 only at commits that create a record | Reconciliation R9 (A1 sign-off) | Text agrees with commit steps 4–8 | design.md §4 ordering of system records |
| F28 | done | P2 | G6 | §11 "Outside the commitment": rows never enter the state trie; whether a per-commit digest does, and who commits it, is open | Reconciliation R10 (A4 sign-off) | §11 agrees with the §4 catalogue (record 5) and D19 | design.md §11 Why Cells Are the Wrong Shape |

## K2 — Spec decisions

Tracks: **Q** query · **L** lifecycle · **H** history, retention, proofs · **E** encoding · **M** metering and assumptions.

Dependencies on K4: the cross-store halves of D03 and D04 wait on P05; D05, D06 and D12 wait on P06;
D18 waits on P07. D09 closes only once the vector format (implementation plan issue 45) exists. If
P05 drops segments from v1, F03, F10 and D17 are deferred with §11.

| ID | Status | Pri | Trk | Decision needed | Sources | Acceptance check | Link |
|---|---|---|---|---|---|---|---|
| D01 | waiting_on_additional_input | P1 | Q | Filter evaluation: predicate combination (conjunction only / ordered DNF / negation / match-all), bounds on groups, predicates, nesting; cost shape. New section between §5 and §12 | F-A1, F-Ariadne, R10 | Section exists; Epic 8 builds from it | decisions_for_architect.md D01 (API already fixes ordered DNF; recommends: negation as ANDNOT within a group, no match-all in v1, caps in `#params`, flat DNF only — `[API]` strike "nesting", canonical cost with region pruning) |
| D02 | waiting_on_additional_input | P1 | L | `rewind(to)`: exists or not; what it undoes (cells, index, tries, history, `#roots`, segments); ordering across MDBX and segments; invalidation of handles, cursors, caches; commit identity after rewind | F-A4, A-D06, R12 | Trace: rewind to 99, commit new 100, no stale cache or handle survives | decisions_for_architect.md D04·D03·D02 (recommends: exists; undo by change-set replay one txn per commit, MDBX then segments; root-based identity for cursors/`at`, new `Stale` error) |
| D03 | waiting_on_additional_input | P1 | L | Crash recovery: restart from `Superblock` head; segment truncation; behaviour on segment-fsync or MDBX-write failure; retry idempotence; issued receipts | A-D06, F-Argus6, R12, R22 | Failure table: crash point × detection × recovery | decisions_for_architect.md D04·D03·D02 (restart procedure; failure-point table; deterministic vs environmental errors, new `Io`) |
| D04 | waiting_on_additional_input | P1 | L | Concurrency contract: one MDBX read snapshot per branch and per query; guard critical section vs. segment appends; arbitration of two sealed candidates | A-D06, Epic 8 | Interleaving trace: no mixed states; losing candidate leaves no durable row | decisions_for_architect.md D04·D03·D02 (recommends: one read txn per operation with head check inside it; commit mutex covering guard → segment fsync → MDBX txn) |
| D05 | waiting_on_additional_input | P1 | H | Retention and historical discovery: retained structures per read class; earliest supported commit; reader/cursor protection from GC; `Pruned` vs `NotFound`; discovery of deleted terms and cell names at T; where a shard's starting mark `mark(S−1)` survives once the previous system-segment shard is pruned (from F03) | A-D07, F-A4/Hermes, R24 | Service matrix; trace across retention boundary yields correct or `Pruned`, never silent empty | decisions_for_architect.md #7 (service matrix; one node-configured window; GC roots = `#roots` in window; discovery via history tables; no cursor lease) |
| D06 | waiting_on_additional_input | P1 | H | Proof scope and non-inclusion witness: proven classes; how the server obtains a mismatching virtual leaf's tagged value at head and historically; cost | A-D04, R verifiability | Four traced proofs (missing slot, prefix mismatch, mismatching leaf, singleton root) plus historical deleted case | decisions_for_architect.md #8 (recommends nested leaf preimage + stored `leaf_value_hashes`; range completeness a non-goal) |
| D07 | waiting_on_additional_input | P1 | L | Branch transitions: delete visibility over real overlay values; "net diff" with restored/no-op entries; history of cancelled changes; create-then-delete in one commit | A-D05 | State inventory per mutation; trace patch→delete→get, delete→rollback, write-back-to-original, seal, reopen | decisions_for_architect.md D07 (recommends: drop deleted-record set, tombstone every cell; three-valued pre-image; net diff = entries ≠ origin) |
| D08 | waiting_on_additional_input | P1 | L | `#recordKeys` on delete: binding survives (§4) or is removed (§10) | F-Heph | §4 and §10 agree; fixture | decisions_for_architect.md D08 (recommends A: removed) |
| D09 | waiting_on_additional_input | P1 | E | Normative encoding profile: Roaring version, container selection and run-opt rule; odd-nibble padding; `EMPTY_ROOT`; `typeTag` for reserved layouts; absent pre-image encoding; `bool` byte forms; shipped type-id map; cross-table key caps | A-D02, R26 | Vectors for each item; decoder rejects non-canonical forms | decisions_for_architect.md #9 (13 items pinned; 10 vector sets) |
| D10 | waiting_on_additional_input | P2 | M | Metering shape (one paragraph) and activation semantics at `A`; minimum install→activation window; behaviour on incomplete weights | F-B9, A-D09, R16 | Timeline install, A−1, A, A+1 with schedule used and `priced_at` | decisions_for_architect.md #11 (import API contract; price by target commit; `#minActivationDelay`; check at install) |
| D11 | waiting_on_additional_input | P2 | M | Cursor contract: cursor in the receipt? `machineId` determinism; fingerprint scope (same-sequence vs same-query, expressed as an exclusion — from the §13 Open box); cursor + offset + `at` precedence | F-Argus2/Hermes, §13 open | API text; determinism statement | decisions_for_architect.md #13 (deterministic cursor, `machineId` out; same-sequence exclusion list; strict precedence) |
| D12 | waiting_on_additional_input | P2 | H | Live paging guarantee narrowed to position-shift anomalies; membership and projection may change; does a pinned cursor lease retention? | A-D10 | Text plus three-record counterexample | decisions_for_architect.md #14 (narrow the claim; no lease) |
| D13 | waiting_on_additional_input | P2 | M | Environment assumptions: MDBX durability/fsync model; who owns RAM caps (overlays, undo logs, sealed candidates, staged rows, warm sequences) | R3, A-D11/Elena | Assumptions section exists; every "bounded by RAM" cites it | decisions_for_architect.md #10 (`SYNC_DURABLE`; no node-local caps on the consensus path; memory formula) |
| D14 | waiting_on_additional_input | P2 | M | Cost qualifications: sort = N fetches + O(N log N) comparisons; warm cache holds keys or refetches; history-bitmap growth from architecture §10 | A-D11 | Table of quantities: stated vs. qualified | decisions_for_architect.md #12 |
| D15 | waiting_on_additional_input | P3 | L | Stale handle: `Conflict` on commit vs `HandleInvalid` elsewhere — intentional? | F-Heph | One sentence | decisions_for_architect.md #16 (recommends one error, `HandleInvalid`) |
| D16 | waiting_on_additional_input | P3 | L | Refuse a second consecutive `rollback()`? | §10 open | API decision | decisions_for_architect.md #17 (recommends: rollback on an empty frame is an error) |
| D17 | waiting_on_additional_input | P3 | E | Typed columns in segments? (full argument carried in design.md Open Questions, from the §11 Open box) | §11 open | Decision-log entry | decisions_for_architect.md #18 (recommends untyped) |
| D18 | waiting_on_additional_input | P2 | H | Second-engine contract: is the §4 reserved-record layout part of the commitment contract or a Golem DB choice? | F-Athena | Statement in normative appendix | decisions_for_architect.md #15 (**normative, by requirement** DI-2/NF-8; alternative withdrawn) |
| D19 | waiting_on_additional_input | P2 | H | Per-commit log digest (SE-1): definition, and whether the host or the engine commits it | requirements SE-1, open question 8; surfaced by P05 | Digest vectors; `#logDigests` cell proof | decisions_for_architect.md #19 (recommends engine commits lag-one in `#logDigests`, id 5) |
| D20 | proposed | P2 | Q | Arkiv lookups by hash: Arkiv keeps no store of its own, and segment rows are reachable only by `(segment, ordinal)`. Which of block-by-hash, transaction-by-hash, receipt-by-transaction and log filtering Arkiv needs, and as which records and attributes. `#rootIndex` serves block-by-hash only if the block hash is `GlobalRoot` | Reconciliation 2026-09-30 (A4 sign-off) | Record/attribute layout per lookup, with its commitment and history cost | design.md §11 The Model; What This Assumes of the Schema |

## K3 — Structure

| ID | Status | Pri | Change | Sources | Acceptance check | Link |
|---|---|---|---|---|---|---|
| S01 | triaged | P1 | Target shape: documentation set (recommended) vs. single file with status lines | R Part 4, A-D12, F-B7 | Decision-log entry | |
| S02 | done | P2 | Reader statement and reading routes (G1) | F-Icarus, A-D12/Sam/Marcus | First screen of the overview | design.md "Who Should Read This, and How" |
| S03 | done | P1 | Status register per chapter: `settled / proposed / open / depends-on-metering`; §1 "settled" → "design record; status per chapter" (G1) | F-B7, A-D12, R33 | Every chapter has a line; every `Open —` box listed | design.md "Status by Chapter" |
| S04 | done | P2 | Glossary; de-overload: trie "branch node" → "interior node"; unqualified "node" = machine; "commit" vs "commitment" (G5) | F-B8, R31 | grep counts; glossary before §1 | design.md Glossary (before §1); 27 rename edits; "branch node" 11→1 (the glossary's own mention), "sub-branch" 3→0 |
| S05 | triaged | P1 | Normative-surface appendix: everything that changes the root if changed, and the explicit list of what is free | F-B10, F-Athena, R26 | One list; D09 vectors reference it | |
| S06 | triaged | P2 | Overview document: goals, non-goals, assumptions (D13), building blocks, component diagram, life of a write, life of a query, invariants | R1–R4, R20 | Rubric set-level checklist passes | |
| S07 | triaged | P1 | Filter-evaluation section (output of D01) | F-A1 | Section exists | |
| S08 | triaged | P1 | Recovery, reorg and failure document (output of D02–D04) with "things that can go wrong" table | R12, R22 | Table exists; each row has detection + handling | |
| S09 | triaged | P2 | Host-surface box at head of branch and segment chapters | F-C11, F-Hermes | One screen | |
| S10 | triaged | P2 | Per-operation semantics table: create/patch/delete/rewind × every structure | R10, A-Tomas | Table exists; agrees with D07 | |
| S11 | triaged | P3 | One running example dataset with valid IDs; one end-to-end fixture | R7, A-R14 | Same records in every worked example | |
| S12 | done | P2 | Consolidated open-questions list with owners; one-line pointers left in place (G5) | F-Prometheus, R33 | List exists | design.md "Open Questions" (end); pointers at §11 typed-columns box and §13 Open Question on Paging; owners/status by reference to this register |
| S13 | triaged | P2 | Testing and conformance strategy document | R27, Epic 6 | Document exists; Epic 6 links it | |
| S14 | triaged | P2 | Information model in one home; mechanism docs reference, not repeat | R6 | No normative duplication | |
| S15 | triaged | P2 | Product-decisions list in the short document | F-C13 | List exists | |

## K4 — Product decisions

| ID | Status | Decision | Sources | Owner | Default if undecided | Link |
|---|---|---|---|---|---|---|
| P01 | decided | Segment set fixed at genesis vs. admin-activatable | F-Prometheus2 | Product + tech lead | Genesis-fixed, as written | **by requirement CS-5** (declared log segments are instance parameters) — decisions_for_architect.md K4 table |
| P02 | done | `#params` immutable with no admin path | F-Prometheus2 | Product + tech lead | Immutable, as written | **by requirement CS-5**; §4 "graduates to the admin class … not designed now" hedge struck, CS-5 cited |
| P03 | decided | Launch type subset chosen from Arkiv's vocabulary | F-Prometheus2 | Product | As written | **by live-product target types** (`arkiv-live-product.md` §5): the §3 bold set matches; SDK owes the `numeric` mapping |
| P04 | done | Arkiv vocabulary inside a "generic" engine vs. the Arkiv-ignorance invariant | F-C12, F-Athena | Product + tech lead | Keep examples; reword the two definitional uses (§8, §11 ¶1) | **by requirement** (genericity as enforcement mechanism); §8 "committed in block headers" → "a host publishes — for a chain, in its block header"; §11 ¶1 "what a receipt is" → "what any row means" |
| P05 | done | §11 segments: adopted or still a proposal | A-D12, F-Argus3 | Tech lead | Proposal; catalogue marks id 4 as proposed | **adopted, by requirement SE-1**. Edits landed: §11 status `recorded, open`; §11 ¶1 "proposes" → "gives … the immutable log of SE-1"; catalogue rows lose *proposed*, id 5 reserved for `#logDigests` (D19); genesis-rigidity sentence cites CS-5. Owed: D19 digest, D05 shard-mark survival |
| P06 | decided | "Full history" promise (§1 property 5) vs. retention window with archival nodes | A-Elena, A-D07 | Product | Qualify property 5 | **by requirement DI-7 / CS-5**: minimum window is an instance parameter; consensus path refuses beyond it on every node; archival surface separate. History's *tier* remains the requirements' own open question 2 (product) |
| P07 | decided | Is a second conformant engine a goal? (decides whether D18 is contract or choice) | F-Athena, A-Marcus | Product | Not a goal for v1; §4 layout is Golem DB's | **yes, by requirement** (swappability; NF-8 commitment vectors; DI-2 engine state under the root) — so the §4 layout is *normative*, the opposite of the programme's default. D18 = A |
| P08 | waiting_on_additional_input | Must the live DSL's standalone `!=` / `!` (negation-only queries) survive into v0.1? | `arkiv-live-product.md` query language; requirements OQ6; surfaced by D01 | Product | Live-set index term adopted (D01 1.1-B/1.2-B): users have the capability today | decisions_for_architect.md *Product questions answerable from the live product*; flips D01 1.1/1.2 and drops one `[API]` change |
| P09 | waiting_on_additional_input | Glob `~`: general glob, prefix-only, or dropped? | `arkiv-live-product.md`; target draft "no glob" | Product | Prefix-only: literal + one trailing wildcard → §5 prefix scan; else `InvalidQuery` | same section; one sentence in D01 |

## Tasks moved from the design text

Filled by F17.

| ID | Status | Task | Origin | Link |
|---|---|---|---|---|
| T01 | open | Verify against reth's source that NippyJar is an append-only columnar container with per-column `zstd`/`zstd-dict`/`lz4` compression, an offset list for random access by row, and a side configuration file; and that it permits single-row decode | §11 On-Disk Form; standing caveat on reth claims in `arkiv-execution-client.md` | design.md §11 |
| T02 | open | Verify against reth's source that static files are appended and fsynced before the MDBX transaction commits, and truncated to the last committed mark on startup | §11 Rejected Alternatives; same caveat | design.md §11 |
| T03 | open | Regenerate `golem-db-design-short.md` from scratch off the final design text (after Phase 3 and the Phase 5 checks): same chapter structure, decisions only, plus the product-decisions list (S15). Until then the short document lags this one and must not be cited as current | Triage 2026-09-25; programme Phase 3 step 14 | |
| T04 | open | Port the "Empty user records are legal" paragraph from branch `matthiaszimmermann/feat/docs-metering` into design.md §3 when that branch is rebased onto the v2 adoption (it conflicts on the `recordID 42 → 100` line it follows) | Reconciliation 2026-09-30 | design.md §3 Record Identity |

## Triage log

- **2026-09-30 (reconciliation).** The v2 text of design.md (all rows above marked done, through
  G5) was compared line by line with the previous design.md at `66e83d6`, and adopted as design.md.
  The six edits that change committed bytes or engine behaviour were signed off individually:
  **F09** (`#alloc` in the net diff, covered by the root; v1 contradicted its own §2, §4 and §10
  overlay table), **F13** (retyping per write, as the API already states), **F08** (a failed call
  applies nothing; frame rollback is the host's decision), **P05 / F10 / D19** (§11 adopted: the API
  already exposes `immutable_data_*`; id 4 and the `#params` fields were already specified in v1's
  §11; id 5 stays reserved for `#logDigests`), **P02 / P01** (no admin path for `#params` or the
  segment set) and **F16** (ABCI 2.0 mapping). All accepted. The comparison produced fix rows
  F19–F28, landed together as G6 in the adoption PR rather than in PRs of at most five, since
  they are one-hunk consistency fixes to the same text. **Arkiv stores all of its state and block
  data in Golem DB** (confirmed 2026-09-30), so §11 is on Arkiv's critical path: new row D20
  (lookups by hash), and a proposed priority raise for T01, T02, D03, D05, D17 and D19, not yet
  applied to their rows. The metering branch's §3 paragraph is outside both texts: T04.
- **2026-09-25 (K4).** P01, P02, P04, P05, P06, P07 resolved against the requirements document
  (`arkiv-source-of-truth/golem-db.md`) rather than by a product meeting: each is answered by a
  cited requirement (CS-5, SE-1, DI-7, DI-2, NF-8, genericity principle). P03 stays defaulted.
  Consequences: §11 is adopted (SE-1) and owes a per-commit digest — new row D19; the reserved
  layout is normative (D18 = A, alternative withdrawn); D05 revised to a `#minRetention` instance
  parameter with a separate archival surface; D02 revised to "semantics specified, feature
  deployment-optional and not in v1" per the requirements' open question 4; D13 admits the one
  node-local cap DI-4 permits (open-branch count). Seven brief/requirement conflicts are tabulated
  in `decisions_for_architect.md`.
- **2026-09-25.** Kinds confirmed as registered; no row contested. K1 (F01–F18) and the Phase 1
  skeleton rows (S02, S03, S04, S12) accepted as a batch. F13 verified against the API: "a `set` may
  change kind and/or type". K4 rows stay with their stated defaults until an owner is named; a
  default applied in text is marked *defaulted, not decided*. S01 (target shape) deferred to the end
  of Phase 2: Phases 1–2 land in the current file under either shape, and the size of D01–D09
  output is the best evidence for the choice.
