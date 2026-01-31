# Missing Signer Validation (Pinocchio)

This repository demonstrates a common authorization vulnerability in Solana programs written with Pinocchio: **missing signer validation**.

The example shows how checking only a public key is insufficient for access control, and how failing to require a signature allows unauthorized state mutation.

---

## Vulnerability Overview

The program manages a simple on-chain configuration account (`ConfigState`) that stores:

- an `admin` public key
- a `protocol_fee` value

There are two instructions:

- `UpdateFeeVulnerable`
- `UpdateFeeSafe`

Both instructions compare the provided admin account’s public key against the stored `admin` field. However, **only the safe version enforces that the admin account actually signed the transaction**.

---

## Root Cause

In Solana, **any account can be passed into an instruction**, regardless of whether its owner approved the transaction.

A public key comparison alone does not prove authorization.

If a program:
- checks `account.key() == expected_pubkey`
- but does NOT check `account.is_signer()`

then **any user can pass the admin’s public key and bypass authorization**.

---

## Vulnerable Pattern

In `UpdateFeeVulnerable`, the program performs only a key equality check:

- It verifies `config.admin == admin.key()`
- It does NOT verify `admin.is_signer()`

This allows an attacker to:
1. Pass the real admin account (not owned or signed by them)
2. Successfully pass the equality check
3. Modify `protocol_fee` without authorization

This is a **pure authorization bypass**, not a cryptographic failure.

---

## Safe Pattern

In `UpdateFeeSafe`, the program adds a required signature check:

- It verifies `config.admin == admin.key()`
- It additionally enforces `admin.is_signer()`

This ensures:
- The admin public key matches
- The admin explicitly approved the transaction

Without both conditions, the instruction fails.

---

## Why This Matters

This vulnerability is especially dangerous because:

- The code “looks correct” at first glance
- Public key checks are commonly misunderstood as sufficient
- The bug does not require complex exploits or race conditions
- Any protocol parameter (fees, authorities, limits) can be modified

This class of issue has appeared repeatedly in real-world audits.

---

## Key Takeaway

**Authorization requires signatures, not just identities.**

When a program relies on an account’s authority:
- Always verify the public key
- Always verify `is_signer`

Missing either check results in broken access control.

---

## Disclaimer

This program is intentionally minimal and insecure.
It is for **educational and auditing purposes only** and must not be used in production.
