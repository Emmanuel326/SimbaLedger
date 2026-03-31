# SIMBALEDGER SPECIFICATION
## Ledger Invariants and System Laws
### Version 0.2.0

---

## 0. PREAMBLE

SimbaLedger is a deterministic financial ledger.

This document defines the invariants of the system. These invariants are
absolute. They are not configuration, not policy, and not subject to runtime
interpretation.

Any implementation that violates these invariants is incorrect.

The implementation may change.
The invariants do not.

---

## 1. CONSERVATION OF VALUE

### 1.1 Master Invariant

    Σ(all account balances) = constant

This holds for all committed states, excluding explicitly defined external
mint or burn operations.

---

### 1.2 Transfer Conservation

For every transfer or atomic batch:

    Σ(debits) = Σ(credits)

No operation may create or destroy value.

---

## 2. TRANSFER MODEL

### 2.1 Primitive

A transfer is defined as:

    transfer(id, debit_account, credit_account, amount)

Constraints:

    debit_account ≠ credit_account
    amount > 0

SimbaLedger uses binary transfers as its primitive. This is a design choice,
not a definition of double-entry accounting. Complex transfers (1→many,
many→1, many↔many) are built by composing this primitive.

---

### 2.2 Atomicity

A transfer is atomic.

    either: both debit and credit are applied
    or:     neither is applied

Partial application is forbidden.

---

### 2.3 Immutability

Once committed:

    - transfers SHALL NOT be modified
    - transfers SHALL NOT be deleted

All corrections MUST be expressed as new transfers.

---

### 2.4 Idempotency

Transfer identifiers are globally unique and persistent.

    process(id) → applied once
    process(id) → subsequent calls return prior result

Idempotency MUST be durable across crashes and restarts.

---

## 3. ACCOUNT MODEL

Each account maintains:

    credits_posted
    debits_posted
    credits_pending
    debits_pending

---

### 3.1 Derived Balances

    available = credits_posted - debits_posted

    total = (credits_posted + credits_pending)
          - (debits_posted + debits_pending)

Balances are derived. They are not independently stored.

---

### 3.2 Pending Semantics

Pending and posted values are disjoint.

State transitions:

    Pending Transfer:
        debits_pending  += amount
        credits_pending += amount

    Post Pending:
        debits_pending  -= amount
        debits_posted   += amount
        credits_pending -= amount
        credits_posted  += amount

    Void Pending:
        debits_pending  -= amount
        credits_pending -= amount

---

## 4. POLICY VS INVARIANT

### 4.1 Ledger Invariants

The following MUST always hold:

    - conservation of value
    - transfer atomicity
    - transfer immutability
    - idempotency

---

### 4.2 Account Policy

Balance constraints are policy, not invariant.

Account behavior is defined by flags:

    enum AccountFlags {
        Normal,           // Cannot go negative
        OverdraftAllowed, // Can go negative up to a predefined limit
        CreditLine,       // Special credit account (e.g., Fuliza)
        System,           // Can go negative (fee accounts, treasury)
    }

Validation rules MAY vary by account type.

---

## 5. BATCH ATOMICITY

A batch of linked transfers is atomic.

    if any transfer fails:
        all transfers fail

    if all succeed:
        all are committed

Partial success is forbidden.

---

## 6. IDENTIFIERS

### 6.1 Account Identity

    - 128-bit identifiers
    - ID 0 is reserved
    - IDs are never reused

---

### 6.2 Transfer Identity

    - globally unique
    - immutable
    - idempotent

---

## 7. TIME AND ORDERING

### 7.1 Timestamps

Timestamps are metadata only.

    - not used for ordering
    - not used for correctness

---

### 7.2 Ordering

Ordering is defined by logical sequence:

    sequence ∈ ℕ (monotonic, crash-safe)

Distributed systems MAY use logical clocks.

---

## 8. ATOMIC EXECUTION MODEL

Atomicity is enforced at the engine level.

The system MUST guarantee:

    - write-ahead logging (WAL)
    - deterministic replay
    - crash recovery to last committed state

No correctness property depends solely on storage behavior.

---

## 9. STORAGE BOUNDARY

The engine does not trust storage.

The engine MUST:

    - validate before write
    - apply state transitions deterministically
    - verify critical assumptions during development (debug assertions)

Storage is an implementation detail, not a source of truth.

---

## 10. FORBIDDEN STATES

The following states are impossible in a correct system:

    - creation of value
    - destruction of value
    - partial transfer application
    - partial batch commit
    - transfer mutation
    - transfer deletion
    - duplicate transfer execution
    - debit without corresponding credit
    - credit without corresponding debit
    - reliance on timestamps for correctness

---

## 11. VERIFICATION

The system MUST be validated using:

    - property-based testing (proptest)
    - crash recovery testing
    - idempotency replay testing
    - batch atomicity testing
    - chaos testing (process kill, disk corruption)

Given identical input, the system MUST produce identical output.

---

## 12. EVOLUTION

Any extension MUST preserve all invariants.

If a feature violates an invariant:

    - it MUST be explicitly defined as an exception
    - it MUST include compensating controls
    - it MUST undergo additional verification

---

## VERSION HISTORY

    0.2.0  2026-04-01  Specification rewrite:
                        - Clarified binary transfers as design tradeoff
                        - Split balance constraints: invariant vs policy
                        - Added transfer immutability
                        - Durable idempotency
                        - Engine-level atomicity with WAL
                        - Timestamps as metadata only
                        - Storage trust boundary
    0.1.0  2026-03-31  Initial draft (retired)

---

The code may change.
The invariants do not.
