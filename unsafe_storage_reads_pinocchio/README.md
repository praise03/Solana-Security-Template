# Unsafe Account Data Reads in Pinocchio (Educational Example)

This program demonstrates **common unsafe patterns** developers use when reading Solana account data in low-level Rust programs built with **Pinocchio**.

The goal is educational: to show how seemingly normal Rust code can become **unsafe or exploitable** when applied to raw on-chain account data.

---

## What This Program Shows

The instruction processor reads raw account data (`&[u8]`) and demonstrates **five unsafe ways** to interpret it as structured data.

Each pattern compiles, and some may even appear to “work”, but all introduce **undefined behavior**, incorrect reads, or security risks.

The code itself contains inline comments explaining:
- Why each pattern is unsafe
- What assumption is being violated
- What kind of real-world bug or exploit could result

---

## Covered Unsafe Patterns

1. **`transmute()` on raw bytes**  
   Assumes perfect alignment and layout. Account data provides neither.

2. **Casting to `#[repr(C, packed)]` structs**  
   Removes padding, causing multi-byte fields like `u64` to be unaligned.

3. **Direct field access on packed structs**  
   Creates unaligned references, which is undefined behavior in Rust.

4. **Assuming alignment for `#[repr(C)]` structs**  
   Even with C layout, account data is just bytes—alignment is not guaranteed.

5. **Using `read_unaligned()` with `#[repr(Rust)]`**  
   Default Rust layout is unstable and must never be used for raw decoding.

---

## Why This Matters

On Solana:
- Account data is attacker-controlled input
- Misreads can lead to:
  - Incorrect authority checks
  - Fake balances or state
  - Unauthorized fund movement
  - Program crashes (DoS)

---

## What This Program Does *Not* Show

- Safe deserialization patterns
- Borsh / Anchor / manual byte parsing best practices

This is intentionally a **“what not to do”** example.

---

## Key Takeaway

> **Account data is untrusted, unaligned bytes.  
> Rust structs do not magically make it safe.**

Always validate layout, alignment, and decoding explicitly.

---
