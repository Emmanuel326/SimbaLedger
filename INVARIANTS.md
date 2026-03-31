# SimbaLedger Invariants

## The Laws of the Ledger

This document defines the **immutable laws** that SimbaLedger enforces. These are not configuration options. They are not negotiable. They are the foundation upon which all financial correctness rests.

---

## 1. Double-Entry Accounting (The Fundamental Law)

### Invariant
Every transfer MUST debit exactly one account and credit exactly one account.

### Enforced By
- `Transfer` struct requires both `debit_account_id` and `credit_account_id`
- Validation rejects transfers with same account (debit = credit)

### What This Prevents
- Money being created or destroyed
- One-sided transfers
- Self-transactions that could be used for fraud or confusion

### State Transition

Account A: balance -= amount
Account B: balance += amount
Total system balance: UNCHANGED


---

## 2. Account Balance Invariants

### 2.1 Available balance

available = credits_posted - debits_posted


**Invariant:** `available` can never be negative after any committed operation.

**Enforced By:**
- `can_debit()` checks before any debit operation
- All transfers validate `debit_account.available_balance() >= amount`

**What This Prevents:**
- Overdrafts (unless explicitly allowed via pending transfers)
- Negative balances in posted amounts

---

### 2.2 Total Balance (Including Pending)

total = (credits_posted + credits_pending) - (debits_posted + debits_pending)


**Invariant:** `total` can never be negative.

**Enforced By:**
- Pending transfers validate `debit_account.total_balance() >= amount`
- Ensures pending amounts don't exceed total funds

**What This Prevents:**
- Over-committing pending transfers
- Creating more pending obligations than funds exist

---

### 2.3 Pending vs Posted Separation

**Invariant:** Pending amounts are separate from posted amounts. Available balance excludes pending.

**State Transition Rules:**

| Operation | Debit Account | Credit Account |
|-----------|---------------|----------------|
| **Pending Transfer** | `debits_pending += amount` | `credits_pending += amount` |
| **Post Pending** | `debits_pending -= amount`<br>`debits_posted += amount` | `credits_pending -= amount`<br>`credits_posted += amount` |
| **Void Pending** | `debits_pending -= amount` | `credits_pending -= amount` |

**What This Prevents:**
- Spending funds that are locked in pending transfers
- Double-counting pending amounts
- Loss of atomicity in two-phase commits

---

## 3. Transfer Idempotency

### Invariant
A transfer with a given `id` can only be processed once. Subsequent attempts return `AlreadyProcessed` without changing state.

### Enforced By
- Idempotency cache mapping `transfer_id` → result
- Check occurs BEFORE any validation or state changes

### What This Prevents
- Double-spending
- Duplicate transaction processing
- Race conditions in retry logic

---

## 4. Two-Phase Commit (Pending/Post/Void)

### 4.1 Pending Transfer States

A pending transfer follows a strict state machine:



### Invariants
1. A transfer cannot be posted or voided before it exists as pending
2. A transfer cannot be posted or voided twice
3. A posted pending transfer cannot be voided
4. A voided pending transfer cannot be posted

### Enforced By
- `post_pending` and `void_pending` look up original pending transfer
- Validation rejects operations on non-pending transfers
- Idempotency prevents duplicate operations

### What This Prevents
- Orphaned pending transfers
- Double-committing or double-cancelling
- Inconsistent two-phase commit state

---

## 5. Linked Events (Batch Atomicity)

### Invariant
Within a batch of linked transfers, either ALL succeed or ALL fail. Partial success is impossible.

### Enforced By
- `linked` flag marks transfers that belong together
- If any linked transfer fails, the entire batch is rolled back

### What This Prevents
- Partial updates in complex operations
- Inconsistent state across multiple accounts

---

## 6. Account Identity

### Invariant
Every account has a unique 128-bit identifier. Account ID 0 is reserved and cannot be used.

### Enforced By
- Account creation rejects ID 0
- All lookups use the full 128-bit range

### What This Prevents
- Account collisions
- Invalid system account references

---

## 7. Transfer Amount Invariants

### 7.1 Amount Must Be Positive


amount > 0


### Enforced By
- `Transfer.validate()` rejects amount == 0

### What This Prevents
- Zero-value transfers (wasteful, often used in attacks)
- Negative amounts (would require sign interpretation)

---

### 7.2 Amount Bounds
amount <= MAX_U64 (18,446,744,073,709,551,615)


### Enforced By
- Rust's `u64` type
- Arithmetic overflow checked in debug builds

---

## 8. Timestamp Invariants

### Invariant
Every transfer has a monotonic timestamp (nanoseconds since UNIX epoch).

### Enforced By
- Default timestamp using system time
- (Future: enforce monotonic ordering for replication)

### What This Prevents
- Out-of-order processing ambiguity

---

## 9. Storage Invariants

### 9.1 Atomic Writes
Account and transfer updates are atomic. Either both accounts update, or neither does.

### Enforced By
- Storage backend implements atomic operations
- Engine updates accounts in a single transaction

### What This Prevents
- Partial updates where one account updates but the other doesn't

---

### 9.2 Crash Recovery
All committed transfers survive process crashes.

### Enforced By
- Storage backend provides durability (fsync, WAL)
- (Future: LSM tree with write-ahead log)

---

## 10. What SimbaLedger Forbids (The "Impossible" List)

The following operations are **impossible** in a correct SimbaLedger system:

| Forbidden Operation | Why It's Impossible |
|---------------------|---------------------|
| Creating money | Double-entry requires debit and credit |
| Destroying money | Double-entry requires debit and credit |
| Negative balance (posted) | `can_debit()` check prevents |
| Over-committing pending | `total_balance()` check prevents |
| Double-spending | Idempotency cache prevents |
| Partial batch commit | Linked events roll back on failure |
| Posting non-existent pending | Pending lookup fails |
| Voiding non-existent pending | Pending lookup fails |
| Posting already-posted pending | Idempotency prevents |
| Voiding already-voided pending | Idempotency prevents |
| Same-account transfer | Validation rejects |
| Zero-amount transfer | Validation rejects |
| Orphaned pending transfers | All pending must be posted or voided (enforced by accounting) |
| Account ID 0 | System reserved |

---

## 11. State Transition Diagram


┌─────────────────────────────────────────────────────────────────────────┐
│ ACCOUNT STATE │
├─────────────────────────────────────────────────────────────────────────┤
│ │
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │ ACCOUNT CREATED │ │
│ │ (id, zero balances) │ │
│ └─────────────────────────────┬───────────────────────────────────┘ │
│ │ │
│ ▼ │
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │ CREDIT POSTED (money received) │ │
│ │ credits_posted += amount │ │
│ │ available_balance = credits_posted - debits_posted │ │
│ └─────────────────────────────┬───────────────────────────────────┘ │
│ │ │
│ ┌────────────┴────────────┐ │
│ ▼ ▼ │
│ ┌─────────────────────────┐ ┌─────────────────────────┐ │
│ │ DEBIT POSTED │ │ PENDING DEBIT │ │
│ │ (money sent) │ │ (funds locked) │ │
│ │ debits_posted += amt │ │ debits_pending += amt │ │
│ │ available -= amt │ │ available unchanged │ │
│ └─────────────────────────┘ └───────────┬─────────────┘ │
│ │ │
│ ┌────────┴────────┐ │
│ ▼ ▼ │
│ ┌─────────────────┐ ┌─────────────────┐ │
│ │ POST PENDING │ │ VOID PENDING │ │
│ │ pending→posted │ │ pending removed │ │
│ │ available -= amt│ │ available same │ │
│ └─────────────────┘ └─────────────────┘ │
│ │
└─────────────────────────────────────────────────────────────────────────┘




---

## 12. Verification

### Property-Based Tests
All invariants are tested with property-based testing (`proptest`):
- Random account creation
- Random transfers
- Balance invariants checked after each operation
- Crash recovery simulated

### Chaos Testing
SimbaLedger passes:
- Process kill during writes
- Disk corruption simulation
- Network partition (with replication)

---

## 13. Adding New Features

Any new feature MUST preserve these invariants. If a feature would violate an invariant, it must be:

1. Explicitly documented as an exception
2. Implemented with compensating controls
3. Subject to additional verification

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 0.1.0 | 2026-03-31 | Initial invariants: double-entry, balance constraints, idempotency, two-phase commit |

---

*These invariants are the law. The code may change. The invariants do not.*




