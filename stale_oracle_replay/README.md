# Non-Canonical Pyth Price Feed Vulnerability (Anchor Educational Example)

This program demonstrates a subtle but real vulnerability when consuming Pyth PriceUpdateV2 accounts in Solana programs.

## Overview

Key points about Pyth PriceUpdateV2:

- PriceUpdateV2 accounts are user-provided and not canonical.
- Anyone can create one by calling the Pyth receiver program directly.
- Standard validation helpers only ensure the price was valid at some point in time and within a max_age window.
- They do **not** guarantee that the price is newer than previously accepted prices.

This can lead to a **stale price replay** attack, where an attacker submits a technically valid but old price to manipulate protocol state.

---

## Vulnerable Instruction

`consume_price_vulnerable` demonstrates the common mistake:

- Validates feed ID.
- Checks max_age using Pyth SDK helpers.
- Uses the price for protocol logic assuming it is the most recent.

**Flaw:** It does not enforce that the price is newer than previously accepted prices.  
An attacker can submit a price older than the one already processed, potentially rolling back the protocol’s state.

---

## Safe Instruction

`consume_price_safe` introduces the critical defense:

- Maintains a protocol-level `last_publish_time` in `OracleState`.
- Requires that any new price has a publish_time strictly greater than the last accepted one.
- Updates `OracleState` before performing any sensitive operations.

This ensures that prices are strictly forward-moving and prevents stale price replay attacks.

---

## Security Takeaways

1. Never assume user-provided Pyth PriceUpdateV2 accounts are canonical.
2. Always track the last publish_time you processed and reject older prices.
3. Validation helpers (feed ID, max_age) alone do **not** protect against replay or rollback attacks.
4. Protocol-level invariants are necessary to guarantee correctness.

---

## Disclaimer

- This is an educational example.
- The program is not production-ready.
