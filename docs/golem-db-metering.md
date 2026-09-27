# Golem DB Metering

Status: draft for discussion, 2026-09-27. Builds on [Golem DB API — Cost and Budget](golem-db-design/golem-db-api.md#cost-and-budget) and [architecture §10](golem-db-architecture.md#10-metering-and-cost). How Arkiv turns these costs into fees is in [Arkiv Fees](arkiv-fees.md).

## Why

Golem DB is a shared resource: anyone who pays can make every node compute and store. Metering sets that price, with four aims:

- **Decentralization:** full-node requirements stay bounded, so community members can run one on normal hardware. Archival nodes are out of scope.
- **Security:** no cheap attacks; no caller obtains more work or storage than they pay for.
- **Adoption:** no needless overcharging.
- **Predictability:** costs can be estimated before submitting.

Metering is therefore deterministic and accurate enough, not measured. Where it approximates, it rounds up: undercharging is an attack surface, overcharging only a cost.

Example attacks prevented:

- Cheap writes that trigger expensive trie updates at commit.
- Long cell names that inflate index keys.
- Value rewrites that shrink live data but grow history.
- Range queries over attributes with many distinct values, which step through many index terms; each term stepped over must be charged.

## Principle

- **Golem DB** assesses what each call costs in compute and storage, with no notion of time. Like Ethereum's `SSTORE`, storage is a one-off cost. Full nodes keep a fixed history window, so that cost covers the write and its bounded history.
- **A host** such as Arkiv builds its own pricing on top, for example adding duration to storage ([Arkiv Fees](arkiv-fees.md)).

## Scope

Metering of Golem DB's read and write calls: the cost model, receipts, budgets and cost schedules. How a host turns costs into fees is out of scope.

## TODO: Review Findings

## Goals

1. **Fair cost:** every call pays, accurately enough, for the compute and storage it causes.
2. **Deterministic:** the same call against the same state costs the same on every instance and implementation.
3. **Configurable history:** an instance keeps either the full history (archival use) or the last n commits (full node). In the latter case, storage cost covers the retained history.
4. **Useful to hosts:** a host like Arkiv gets what it needs to build its own pricing on top, such as time-based storage.
5. **Bounded deletion:** removing a record never costs more than a maximum computable at any time from its shape and the current schedule.
6. **Evolvable:** pricing can follow hardware, usage and database growth, with every instance applying a change at the same commit.

## Requirements

Cost properties, from [architecture §10](golem-db-architecture.md#what-cost-must-be):

- **R1 Deterministic:** identical on every implementation, machine and version, for the same call against the same state.
- **R2 Per call and additive:** a cost is attributable to exactly one call and summable across calls.
- **R3 Bounded pricing:** determining a price never takes unbounded work. Write cost may depend on the call's arguments and on state the call reads anyway, never on a read made only for pricing (the no-probe rule).
- **R4 Rounds up:** no caller obtains unbounded work for bounded cost. Where cost is approximated, it is approximated upward: undercharging is an attack surface, overcharging only inefficiency.
- **R5 Defined, not measured:** cost counts logical work (rows, index terms, trie paths) as the reference execution performs it, never physical events (pages, cache hits, timing). A warm and a cold instance charge the same.
- **R6 No refunds:** rollback is free and refunds nothing; commit is free because every call has already paid its share.

Budget:

- **R7:** every call accepts a budget. Exceeding it aborts with `OutOfBudget{spent}` and no partial results; `spent` includes the reads that established the price. A write that runs out of budget also reports the cost it would have required.

Coverage and reporting:

- **R8:** every read and write call (`create`, `get`, `patch`, `delete`, `query`, `count`) is metered for compute, and writes also for storage.
- **R9:** every receipt reports the call's cost and the commit that priced it (`priced_at`), and optionally details of cell and index data added and removed by write calls.
- **R10:** every user record has a maximum deletion cost, computable at any time from its shape and the current schedule. Actual deletion may cost less.

Cost schedules:

- **R11:** a cost schedule is a metering model plus its weights. The model (cost structure (D1) and counting rules) is code identified by a version; weights are committed data, one price per weight name.
- **R12:** weights are adjustable at runtime through write calls, without upgrading Golem DB, taking effect at a defined commit so all instances compute the same cost. A call is priced with the schedule at its branch's base commit and never re-priced.

## Record Model

Golem DB holds two kinds of records ([design §4](golem-db-design/golem-db-design.md#record-classes-and-the-reserved-catalogue)):

- **System records** hold Golem DB's own configuration and state: its deployment parameters (`#params`), the key bindings (`#recordKeys`), and its cost schedules and weights (`@meteringModel`, `@modelWeight`; the design calls these admin records). They are created at genesis, written only by Golem DB, and never deleted.
- **User records** are managed by the application that uses Golem DB, for example Arkiv's entities and accounts. The write calls metered in D1 act on user records.

A record is implemented as a flat list of cells. Each cell is a key-value pair:

- **Key:** the record's ID prefix (`u64`, 8 bytes) followed by the cell name.
- **Value:** a type tag (1 byte) followed by the encoded value.

Every user record has two system cells:

- `#key`: maps the record to its record key.
- `#meta`: the record's metadata, such as its cell counts (D5).

Both have fixed-length encodings. A record without user-defined cells is still two cells.

The reverse mapping, record key → record ID, lives in the system record `#recordKeys`: one binding cell per live record, keyed by the 32-byte record key, holding the record ID.

- `create` adds the binding; `delete` removes it. The removed binding stays in history for the retention window.
- A deleted key can be re-created; it gets a fresh binding and a new record ID. A host that must prevent key reuse enforces it above Golem DB, for example through derived keys.
- "No record with key K" is a non-inclusion proof of the binding.
- `#recordKeys` grows with the number of live records. Per-record caps (D5) apply to user records only.

## D1. Write Cost Model

Meets R1, R2, R3, R7, R11.

A write call (`create`, `patch`, `delete`) touches one record. It runs in two phases: first plan the call and accumulate its cost, then apply it.

1. **Plan** (reads only, no writes):
   1. Admission: check the input's form, cell names and value lengths. No state is read.
   2. Read the key binding (`w_rec[op]`). `create` fails with `AlreadyExists` if it exists; `patch` and `delete` fail with `NotFound` if it does not. A deleted record has no binding, so its key can be re-created.
   3. `patch` and `delete` only: read `#meta` for the current counts. A `create` starts from zero.
   4. `patch` and `delete`: read every touched cell. All operations: read every touched index term. The results decide each operation and its bytes.
   5. Compute the resulting counts and check them against the per-record caps (D5).
   6. Compare the total cost (reads performed + planned writes) with the budget. If it exceeds the budget, fail with `OutOfBudget{spent, required}`: `spent` = the reads performed, `required` = the total. The reads' own cost is tracked as they happen; if it alone reaches the budget, the call stops at that point.
2. **Apply:** execute the planned writes, reusing the phase-1 reads. No metering or validation failure can occur here; only local storage faults remain, which are not cost questions.

A failure in the plan phase writes nothing, so it is charged no write cost (D7). A write is never partly applied.

The plan phase yields the call's cell and index operations, system cells included. The call's cost is the record base cost plus the cost of each of these operations:

```
record op cost = w_rec[op]
               + Σ cell ops   ( w_cell_read + w_cell[create | update | delete]
                              + cell_trie_depth × w_cell_trie_update
                              + bytes written × w_cell_write_byte )
               + Σ index ops  ( w_idx_read + w_idx[join | leave]
                              + index_trie_depth × w_index_trie_update
                              + w_idx_term_create    (only if the term is new; covers its later removal)
                              + bytes written × w_idx_write_byte )

batch cost     = Σ record op costs
```

Every cell and index operation rewrites one leaf's trie path, priced at the modeled depth (D3). `w_cell_read` is charged only where a cell is read: by `patch` and `delete`, never by `create`.

**Record base cost.** `w_rec[op]` covers the fixed per-record work that is not a cell operation:

- All operations: admission, validation and reading the key's binding in `#recordKeys`.
- `create`: increment the record ID counter. A key collision fails with `AlreadyExists` (D7).
- `patch`: check the caps.
- `delete`: one seek to enumerate the record's cells.

This work is the same for every call and touches no trie path, so it folds into one weight per operation with no depth term.

**System cell operations.** The system cells (Record Model) are charged as cell operations, like user cells:

| Record operation | System cell operations |
| --- | --- |
| `create` | create binding, create `#key`, create `#meta`; no reads, the record is new and the binding read is in `w_rec` |
| `patch` | read and update `#meta` |
| `delete` | delete binding (read in `w_rec`), read and delete `#key`, read and delete `#meta` |

System cells have fixed-length encodings, so their byte terms are constants. They are not included in `#meta`'s counts or the receipt's user-cell counts (D4).

**Cell operations.** On `create`, every user cell is a cell create, without a read: the record is new. On `patch` and `delete`, a read of each touched user cell decides the operation:

| Read result and request | Cell operation | Bytes written | Bytes deleted |
| --- | --- | --- | --- |
| cell missing, value assigned | create | new cell | – |
| cell present, value assigned | update | new cell | old cell |
| cell present, deletion requested | delete | – | old cell |

- `w_cell[op]` covers the cell row, its history entry and its change-set entry; the cell-trie path is the depth term.
- Reading and copying an old value into history grows with its size. The write byte weight pre-pays it: every byte is copied into history at most once, when it is overwritten or deleted, so it is charged once, when written (D4). Bytes deleted are reported, not charged.
- A record `delete` performs a cell delete for every user cell.
- Assigning a cell its current value is charged as an update (and, for an indexed cell, as leave + join). An implementation may skip the write; optimizing such calls is the caller's responsibility, not Golem DB's.

**Index operations.** For every indexed cell the call touches, a read of its index term decides the operation:

| Read result and request | Index operation | Charged |
| --- | --- | --- |
| term exists, record joins | join | `w_idx_read + w_idx[join]` |
| term missing, record joins | create term, then join | `w_idx_read + w_idx_term_create + w_idx[join]` |
| record leaves, whether or not it is the last member | leave | `w_idx_read + w_idx[leave]` |
| value or type changes | leave the old term, join the new one | both |

- `w_idx[join]` and `w_idx[leave]` cover the membership change, the posting-list container and the posting-list path; the index-trie path is the depth term.
- `w_idx_term_create` covers both creating a term and its later removal: the term row and the leaf insert and removal. It has no depth term, because the join that follows and the eventual leave already pay for the leaf's path. Removing a term emptied by its last member costs nothing at that point, so every leave costs the same and deletion stays predictable.
- A join writes the term's bytes (D4); a leave deletes them.

**Common rules.**

- Every branch is decided by a read the operation owes anyway, so cost is known before any change is applied (R3).
- Compute and storage are summed, not multiplied: a trie path rewrite costs the same for a 4-byte or a 400-byte value, while writing a value is linear in its length.
- A batch, such as a host transaction, costs the sum of its record op costs. Reads are metered separately (D6).
- Repeated touches of the same record or cell are charged per touch. An implementation may merge the work, never the charge.
- These weights aggregate [architecture §10](golem-db-architecture.md#structural-op-classes)'s finer op classes, which remain the calibration basis (D9).
- On request, a receipt includes the per-part counts as a diagnostic ledger.

## D2. Where Metering Happens

Meets R1, R5.

Cost is computed over Golem DB's logical schema (cell rows, index terms, posting-list containers, trie paths), not over MDBX pages. Page-level work depends on each node's physical history: open read transactions, compaction and configuration change which pages a write touches. Two honest instances would disagree ([architecture §10](golem-db-architecture.md#where-metering-happens)).

Consequences:

- A storage layout change re-measures prices; it never changes the cost structure.
- Cost counts the reference execution, whatever an implementation short-circuits: cached trie nodes and held query results never reduce a charge.

## D3. Write Metering and Modeled Trie Depth

Meets R3, R4.

Trie work is deferred to commit and runs once over the branch's net changes, so a write's physical trie cost depends on the rest of the batch. Cost therefore counts **one modeled path rewrite per distinct leaf touched** in each affected trie, at a modeled depth ([architecture §10](golem-db-architecture.md#deferred-work-and-modeled-paths)). Write cost is computed from the call's arguments and the reads it owes anyway (D1), so a write can be refused before any change is applied.

In D1, these modeled path rewrites are the depth terms of every cell and index operation.

Golem DB keeps committed counters of live cells and distinct index terms, and derives the modeled depth from a pinned table. Tries are 16-ary, so depth grows as ⌈log₁₆ N⌉:

| Population N | Modeled depth |
| --- | --- |
| ≤ 16⁴ (65,536) | 4 |
| ≤ 16⁵ (1,048,576) | 5 |
| ≤ 16⁶ (16,777,216) | 6 |
| > 16⁶ | 7 |

Depth is capped at 7. ⌈log₁₆ N⌉ would reach 8 only above 16⁷ (≈268M) leaves in one trie, which the current architecture is very unlikely to reach.

- The cell trie uses the live-cell count; the index trie uses the distinct-term count. The posting-list trie keeps a constant modeled depth, at the low-cardinality envelope of [architecture §10](golem-db-architecture.md#deferred-work-and-modeled-paths).
- Path-rewrite cost = `cell_trie_depth × w_cell_trie_update` or `index_trie_depth × w_index_trie_update`, using the counters at the branch base.
- The write already knows whether it adds or removes a cell or term, so counters need no extra read (R3).
- Database growth leaves the weights: they reflect cost per node only.
- The model over-charges repeated writes to one cell and writes sharing trie prefixes; it under-charges paths deeper than modeled by a small bounded factor, absorbed by calibrating upward (R4).

## D4. Storage and Size Counting

Meets R8, R9.

**Pay once for every byte made live.** Overwritten and deleted values move into history rather than disappearing, so charging a value's bytes once, when written, pre-pays both copying them into history later and their residency there ([architecture §10](golem-db-architecture.md#the-byte-term-pay-once-for-every-byte-made-live)). Each byte is copied at most once, so the pre-payment is exact for bytes that are later overwritten or deleted, and an over-charge for bytes that never are: the safe direction (R4). Shrinking a value still costs the new value's bytes; deleting costs no bytes. Where an instance retains only the last n commits, residency is shorter than paid for: the safe direction.

**Size per record, names included.** Golem DB stores cell names in full in every cell key and every index term key. Size is counted per record, never shared:

- A cell counts `8 + |name|` bytes for its key (ID prefix and name) and `1 + |value|` bytes for its value (type tag and value).
- An index entry counts `|name| + 2 + |value|` bytes: its term key, name ‖ `0x00` ‖ type tag ‖ value.
- Four counts per record: cells, cell bytes, indexed cells, and index bytes. They cover user cells only; system cells are fixed-size and charged separately (D1).
- Write receipts report cells created, updated and deleted; index joins, leaves and terms created; and cell and index bytes written and deleted, on this basis (R9). Deleted counts are reported, never refunded (R6).
- Counting per record overcounts popular index terms: the safe direction (R4).

## D5. Record Shape and Deletion Bound

Meets R10.

By D1, deleting a record costs:

```
delete cost = w_rec[delete]
            + system cell deletes                        (binding, #key, #meta; fixed size)
            + cells         × (w_cell_read + w_cell[delete] + cell_trie_depth  × w_cell_trie_update)
            + indexed cells × (w_idx_read  + w_idx[leave]   + index_trie_depth × w_index_trie_update)
```

Every leave costs the same, whether or not it empties the term (D1), and deleted bytes were pre-paid at write (D4). So the cost depends only on the record's counts and is exact at current weights and depth.

- `#meta` (Record Model) holds the four D4 counts.
- Updated in the same write that changes any count, as a system cell operation (D1).
- Stores counts, not cost: counts are exact and independent of weights and D3 depth. The maximum deletion cost is computed from them at current weights and depth. Actual deletion may cost less, for example after the database shrinks.
- System cells are not counted in `#meta`; their deletes are fixed-size system cell operations (D1).
- Golem DB's ceilings are deployment parameters in `#params`, fixed at genesis: `#maxCellNameLen`, `#maxStrLen`, `#maxBytesLen`, and caps on cells and indexed cells per user record. System records are exempt from the cell caps; they are never deleted.
- After every `create` and `patch`, the record's resulting counts must stay within the caps. The plan phase checks this (D1); a violation fails the call and writes nothing.
- The cell caps bound a record's maximum deletion cost; the length ceilings bound the size of each write.
- Caps are on counts and lengths rather than cost, so a weight increase cannot push existing records over a limit.

## D6. Read Metering

Meets R5, R8.

Reads are counted, not modeled: nothing on the read path is deferred, so cost accumulates as the work happens and the call aborts when it crosses its budget ([architecture §10](golem-db-architecture.md#read-metering)).

- The counted descent is the reference one, whether or not an implementation short-circuits it (D2).
- Sort comparisons are the exception: modeled as `⌈N log₂ N⌉ × S` from the match count N and S sort terms, so the choice of sort algorithm stays out of the receipt.
- Resolving an item at a past commit is a flat surcharge, independent of how far back.
- Range scans charge every index term stepped over, so ranges over attributes with many distinct values pay for their width.
- Golem DB provides a budgeted, index-ordered scan for bulk deletion, such as a host's expiry purge ([mapping §6](arkiv-golem-db-mapping.md#purge-before-transactions)).

## D7. Budget, Rollback and Commit

Meets R6, R7.

- Cost is charged at the call, against its budget, on the branch it names.
- `OutOfBudget{spent}` reports cost already incurred, including the reads that established the price. A refusal is not free. A write also reports `required`, its full cost, because the plan phase knows it (D1); a read aborts as it goes and cannot.
- Any cost computation that overflows is treated as `OutOfBudget`.
- **Failed writes** are charged for the work done in the plan phase (D1), never for writes:

  | Failure | Charge |
  | --- | --- |
  | Admission (malformed input, `Reserved`) | 0 |
  | Key failure (`AlreadyExists`, `NotFound`) | `w_rec[op]` |
  | Later check (cap exceeded, invalid value) | `w_rec[op]` + the cell and index reads performed |
  | `OutOfBudget{spent, required}` | `spent`: `w_rec[op]` + the reads performed, never more than the budget; `required` reports the full cost |

  `w_rec[op]` includes work a failed call never reaches, such as the record ID counter; that difference is the penalty for a failed call. Returning a receipt with an error code costs nothing extra.
- Rollback is free and refunds nothing: it undoes work already paid for. A receipt is a return value, not state, so a later rollback cannot revoke it.
- Commit is free because it is pre-paid: modeled per-call charges collect the deferred merkleization work in advance.
- Removing an index term emptied by its last member is free for the same reason: the term's creation paid for it (D1).

## D8. Cost Schedules

Meets R11, R12.

- **Model = code.** Cost structure, counting rules, byte definitions and expected weight names, identified by `modelVersion` ([design §4](golem-db-design/golem-db-design.md#meteringmodel-recordid-32)). D1's structure, D3's depth table and D4's counting are model changes: a new version.
- **Weights = data.** One `u64` per weight name per model version, stored in `@modelWeight` and versioned by Golem DB's own history.
- **Install, then activate.** A new model version is installed with its weights and an activation commit ahead of the head; completeness is checked at activation. At most one model is pending.
- **Patch the active model.** Weight changes take effect at the next commit.
- **Priced at branch base.** A call uses the schedule at its branch's base commit, recorded as `priced_at`.
- **Pre-paid work is not re-priced.** Work paid in advance keeps the schedule of the call that paid it: term removal (`w_idx_term_create`), copying bytes into history (write byte weights) and commit. Re-pricing it after a weight change is impractical; if weights rise, the later work is underpaid, bounded to one term removal per term and one copy per byte. Accepted.
- **Authorization is the host's.** Golem DB validates the lifecycle; the host decides who may change weights. A host that changes weights inside its own commits needs branch-scoped admin calls ([mapping §8](arkiv-golem-db-mapping.md#branch-scoped-administration-required-api-extension)).

## D9. Golem DB Weights

| Weight | Used for | Calibrated from [architecture §10](golem-db-architecture.md#structural-op-classes) op classes |
| --- | --- | --- |
| `w_rec[create]`, `w_rec[patch]`, `w_rec[delete]` | Record base cost: admission, binding read, record ID counter, cap check, cell enumeration (D1) | `key_resolve`, `record_create` (allocator part) |
| `w_cell_read` | Read of a touched cell before its operation (D1) | `cell_read` |
| `w_cell[create]`, `w_cell[update]`, `w_cell[delete]` | Cell compute (D1) | `cell_write` / `cell_remove`, `cell_history_append`, `cell_changeset_write` |
| `w_cell_trie_update` | Cell-trie path, per node × `cell_trie_depth` (D1, D3) | `celltrie_path_rewrite` |
| `w_cell_write_byte` | Cell storage: bytes written, including their later copy into history (D1, D4) | byte term, plus the change-set copy |
| `w_idx_read` | Read of an index term before join or leave (D1) | `index_seek` |
| `w_idx[join]`, `w_idx[leave]` | Index membership change (D1) | `index_term_flip`, `secidx_container_rewrite`, `secidx_path_rewrite` |
| `w_idx_term_create` | Creating a new term, including its later removal (D1) | new index row and index-trie leaf, plus their removal |
| `w_index_trie_update` | Index-trie path, per node × `index_trie_depth` (D1, D3) | `indextrie_path_rewrite` |
| `w_idx_write_byte` | Index storage: bytes written (D1, D4) | byte term (new term keys) |
| Read weights (`key_resolve`, `cell_read`, `index_seek`, `index_scan_step`, `sort_compare`, `bytes_read`, …) | Reads, including scans (D6) | read op classes |

Not weights: the D3 depth table (code, metering model version) and the D5 per-record caps (deployment parameters).

## Worked Example: Create, Patch, Delete

One record followed through `create`, `patch` and `delete`, priced by D1. It checks that the spec is complete enough to price every step; the gaps it found are T10–T13.

**Weights.** Illustrative, not calibrated. Both tries sit at modeled depth 6 (about 10M cells and 10M terms), so every trie path costs 6 × 100 = 600.

| Weight | Value | Weight | Value |
| --- | --- | --- | --- |
| `w_rec[create]` | 300 | `w_idx_read` | 100 |
| `w_rec[patch]` | 200 | `w_idx[join]`, `w_idx[leave]` | 650 |
| `w_rec[delete]` | 250 | `w_idx_term_create` | 400 |
| `w_cell_read` | 100 | `w_index_trie_update` | 100 per node |
| `w_cell[create]`, `[update]`, `[delete]` | 350 | `w_idx_write_byte` | 2 |
| `w_cell_trie_update` | 100 per node | | |
| `w_cell_write_byte` | 2 | | |

**The record.** A 32-byte record key and three user cells. Bytes follow D4.

| Cell | Kind, type | Cell bytes (key + value) | Index bytes |
| --- | --- | --- | --- |
| `owner` | attribute, `bytes20` | (8 + 5) + (1 + 20) = 34 | 5 + 2 + 20 = 27 |
| `status` = "active" | attribute, `str` | (8 + 6) + (1 + 6) = 21 | 6 + 2 + 6 = 14 |
| `payload` | field, `bytes` (256 B) | (8 + 7) + (1 + 256) = 272 | – |
| binding in `#recordKeys` | system | (8 + 32) + (1 + 8) = 49 | – |
| `#key` | system | (8 + 4) + (1 + 32) = 45 | – |
| `#meta` (four `u64` counts) | system | (8 + 5) + (1 + 32) = 46 | – |

### Create

The key has no binding. The term `owner` = 0x71C7… is new; the term `status` = "active" already has other members.

| Operation | Terms | Cost |
| --- | --- | --- |
| Record base | `w_rec[create]`: admission, binding read (absent), record ID | 300 |
| Create binding | 350 + 600 + 49 × 2 | 1,048 |
| Create `#key` | 350 + 600 + 45 × 2 | 1,040 |
| Create `#meta` | 350 + 600 + 46 × 2 | 1,042 |
| Create `owner` | 350 + 600 + 34 × 2 | 1,018 |
| Create `status` | 350 + 600 + 21 × 2 | 992 |
| Create `payload` | 350 + 600 + 272 × 2 | 1,494 |
| `owner` joins a new term | 100 + 400 + 650 + 600 + 27 × 2 | 1,804 |
| `status` joins "active" | 100 + 650 + 600 + 14 × 2 | 1,378 |
| **Total** | | **10,116** |

No cell reads: the record is new. Receipt: 3 cells created, 2 index joins, 1 term created, 327 cell bytes and 41 index bytes written. `#meta` afterwards: 3 cells, 327 cell bytes, 2 indexed cells, 41 index bytes.

### Patch

`status` changes from "active" to "closed" (the term "closed" already exists), and `payload` shrinks to 100 bytes: (8 + 7) + (1 + 100) = 116 cell bytes.

| Operation | Terms | Cost |
| --- | --- | --- |
| Record base | `w_rec[patch]`: binding read, cap check | 200 |
| Read and update `#meta` | 100 + 350 + 600 + 46 × 2 | 1,142 |
| Update `status` | 100 + 350 + 600 + 21 × 2 | 1,092 |
| Update `payload` | 100 + 350 + 600 + 116 × 2 | 1,282 |
| `status` leaves "active" | 100 + 650 + 600 | 1,350 |
| `status` joins "closed" | 100 + 650 + 600 + 14 × 2 | 1,378 |
| **Total** | | **6,444** |

Receipt: 2 cells updated, 1 index leave, 1 index join; 137 cell bytes written and 293 deleted; 14 index bytes written and 14 deleted. Deleted bytes are reported, not charged. `#meta` afterwards: 3 cells, 171 cell bytes, 2 indexed cells, 41 index bytes.

**The same patch with a budget of 5,000** fails in the plan phase with `OutOfBudget{spent: 700, required: 6,444}`. `spent` is `w_rec[patch]` (200) plus the reads performed: `#meta` (100), two cells (200) and two index terms (200). Nothing is written.

### Delete

`owner` is the only member of its term, so the term is removed; its removal was paid at creation. "closed" keeps other members.

| Operation | Terms | Cost |
| --- | --- | --- |
| Record base | `w_rec[delete]`: binding read, cell enumeration | 250 |
| Delete binding | 350 + 600 (read in `w_rec`) | 950 |
| Read and delete `#key` | 100 + 350 + 600 | 1,050 |
| Read and delete `#meta` | 100 + 350 + 600 | 1,050 |
| Read and delete `owner`, `status`, `payload` | 3 × (100 + 350 + 600) | 3,150 |
| `owner` leaves; term emptied and removed | 100 + 650 + 600 (removal pre-paid) | 1,350 |
| `status` leaves "closed" | 100 + 650 + 600 | 1,350 |
| **Total** | | **9,150** |

Receipt: 3 cells deleted, 2 index leaves; 171 cell bytes and 41 index bytes deleted, reported only.

### What the Example Shows

- **D5 holds.** From `#meta` alone (3 cells, 2 indexed cells), D5 gives 250 + 3,050 + 3 × 1,050 + 2 × 1,350 = 9,150, the delete's actual cost. Byte counts are not needed.
- **Structure dominates bytes.** The 256-byte payload adds 544 of the create's 10,116 (about 5%). The one-off byte charge is small by design; a host that prices storage over time adds that on top.
- **Deleting costs about as much as creating** (9,150 against 10,116), as architecture §10 predicts: every row, trie path and index entry has to be touched either way.
- **An index value change dominates a small patch.** Changing `status` costs 3,820, of which 2,728 is the index leave and join: two terms, two trie paths.
- **A record's fixed cost** is 3,430 on `create` (base plus three system cells) and 3,300 on `delete`, before any user cell.
- **Gaps found:** T10–T13.

## Open Questions

1. **Is the D3 depth table worth it?** Realistic depths span 4–7 nodes, more likely 4–6. Is that range worth the extra code and a more complex pricing story for users, compared with one fixed depth folded into the weights?
2. **Depth table as code or weights?** [Design §4](golem-db-design/golem-db-design.md#meteringmodel-recordid-32) expresses tunable constants such as a modeled depth as named weights. Should D3's thresholds and depths be weights, tunable without a new model version?
3. **Optional receipt details.** Does "optionally" in R9 mean opt-in per call, or that an implementation may omit them? Arkiv needs them ([Arkiv Fees](arkiv-fees.md) A1, A3).
4. **Full-node history window.** How many commits do full nodes retain? It bounds full-node storage and calibrates the write byte weights.
