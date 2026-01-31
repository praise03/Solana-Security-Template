# Arithmetic Overflow / Underflow (Anchor)

This project demonstrates unchecked arithmetic vulnerabilities in Solana programs and a safe alternative.

Two programs are included:

- arithmetic_vulnerable
- arithmetic_fixed

---

## Vulnerability

The vulnerable program performs raw arithmetic on `u64` values:

- Addition can overflow and wrap to zero
- Subtraction can underflow and wrap to a very large value

Rust does not panic on overflow in release builds, so these bugs are silent.

---

## arithmetic_vulnerable

Uses unchecked arithmetic:

- `vault.balance += amount`
- `vault.balance -= amount`

This allows balance manipulation through overflow or underflow.

---

## arithmetic_fixed

Uses checked arithmetic:

- `checked_add`
- `checked_sub`

Execution fails if an overflow or underflow would occur.

---

## Purpose

This code is intentionally minimal and educational.
It is not production-ready and should not be deployed.

---
