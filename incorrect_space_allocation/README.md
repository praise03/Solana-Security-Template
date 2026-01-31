# Incorrect Account Space Allocation in Anchor

## Overview

This program demonstrates a common and still-prevalent Solana vulnerability: **manually calculating PDA account space instead of using Anchor’s derived space utilities**.

Despite Anchor providing safe, compiler-assisted mechanisms for account sizing, many programs continue to hard-code space values. These calculations are frequently incorrect and introduce long-term protocol risk.

This repository contrasts a **vulnerable manual allocation pattern** with a **safe, derive-based approach**.

---

## Background

Every Anchor account requires an exact byte size at initialization. This size must include:

- The 8-byte account discriminator
- All serialized fields
- Proper alignment
- Any future structural changes

When developers manually calculate account space, mistakes are common and often subtle. These bugs may not surface immediately and can survive audits if no edge cases are triggered.

---

## Vulnerable Pattern

The vulnerable instruction initializes a PDA using a hard-coded `space` value.

This pattern typically fails due to one or more of the following:

- Forgetting the 8-byte discriminator
- Miscounting field sizes
- Incorrect assumptions about layout or alignment
- Structs evolving over time without updating space
- Copy-pasted constants reused across accounts

The result is **under-allocated accounts**.

Initially, the instruction may succeed. However, once the account is fully serialized or modified later, the program can panic, corrupt data, or permanently brick the PDA.

---

## Safe Pattern

The safe instruction uses Anchor’s `#[derive(InitSpace)]` macro and the generated `INIT_SPACE` constant.

This ensures that:

- The discriminator is always included
- Field layout is computed correctly
- Alignment is handled automatically
- Future struct changes do not silently break the program

This approach removes human error from account sizing entirely.

---

## Security Impact

Incorrect account space allocation can lead to:

- Runtime panics during serialization or deserialization
- Permanent PDA corruption
- Unrecoverable protocol state
- Instruction-level or protocol-level denial of service
- Upgrade-induced failures long after deployment

In many cases, once a PDA is under-allocated, it cannot be repaired without migration logic or a full protocol reset.

---

## Key Takeaway

Manually calculating account space in Anchor is an avoidable risk.

If a program is still hard-coding `space = ...` values for PDAs instead of using derived space, it is almost certainly fragile and unsafe under upgrades.

Derived space should be the default approach for all Anchor programs.

---

## Intended Use

This code is provided for educational and security-review purposes to help developers and auditors recognize and eliminate a class of real-world Solana vulnerabilities.
