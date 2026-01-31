# Seed Collision Vulnerability (Anchor)

This repository demonstrates the seed collision vulnerability
in Solana programs using Program Derived Addresses (PDAs).

The code is educational only and is not intended for production use.

---

## What Is Seed Collision?

A seed collision occurs when two different logical accounts
derive the same PDA address because their seed schemes overlap.

If two instructions or account types use the same seeds,
they will resolve to the same PDA, even if they are meant
to represent completely different state.

---

## Why This Is Dangerous

Seed collisions often leads to:

- One account overwriting another
- Unauthorized access to state
- Broken invariants
- Confusing and exploitable behavior

---

## Vulnerable Pattern

Using user-controlled or ambiguous seeds without
a fixed prefix, for example:

    seeds = [user_id]

If another instruction elsewhere uses the same pattern,
both derive the same PDA.

This is especially dangerous in large protocols
with many account types.

---

## Correct Pattern (Domain Separation)

Always include a hardcoded, unique prefix in your seeds:

    seeds = [b"user_state", user_id]

The prefix acts as a namespace for the account type.

This guarantees that:
- Different account types cannot collide
- Future code cannot accidentally reuse the same PDA
- Auditors can reason about address derivation safely

---

## Audit Rule of Thumb

Every PDA seed scheme should answer this question:

"Could another account type ever reuse these seeds?"

If the answer is yes, the scheme is unsafe.

---

## One-Line Rule

Every PDA must have explicit domain separation.

---

## Disclaimer

This repository exists solely for educational and auditing purposes.
It is intentionally simplified to highlight the vulnerability clearly.
Do not deploy this code as-is.
