# Direct Lamport Access CPI DoS

This repository demonstrates a subtle Solana runtime vulnerability commonly referred to as **Direct Lamport Access CPI DoS**.

The bug has appeared multiple times in production protocols because the affected code often looks correct during review and follows otherwise sound patterns.

---

## Summary

A program:
1. Mutates lamport balances directly on one or more accounts
2. Performs a CPI afterward
3. Fails to include all lamport-mutated accounts in the CPI account list

As a result, the Solana runtime aborts execution, creating a denial-of-service condition.

---

## Root Cause

At the start of a transaction and at every CPI boundary, the Solana runtime enforces lamport conservation.

Key detail:
- During a CPI, the runtime only checks lamport balances for the accounts passed into that CPI.
- If lamports were transferred directly between accounts earlier in the instruction, and one side of that transfer is not included in the CPI, the runtime detects an imbalance and throws an error.

This is not a logic bug in the program.
It is a runtime invariant violation.

---

## Vulnerable Pattern

A common real-world pattern looks like this:

1. Directly debit lamports from account A
2. Directly credit lamports to account B
3. Perform a CPI that includes B but not A

Even though total lamports are conserved globally, the CPI-local balance check fails because account A is missing.

---

## Impact

- Any user can trigger the failure
- The instruction becomes uncallable
- This results in a protocol-level denial of service
- No funds are stolen, but critical functionality can be permanently blocked

---

## Fix

The fix is simple and deterministic:

**Include every account whose lamport balance was modified via direct lamport access in all subsequent CPI calls, even if the CPI does not logically use them.**

In Anchor, this is typically done by passing the accounts as `remaining_accounts` in the CPI context.

---

## What This Repository Contains

- A vulnerable instruction demonstrating the failure
- A safe instruction demonstrating the fix
- Extensive inline comments explaining the runtime behavior
- A minimal setup focused on education, not production deployment

---


## Disclaimer

This code is for educational and auditing purposes only.
It is not intended for production use without proper review and adaptation.
