# Frontrunning via Mutable Order State (Anchor)

This repository demonstrates a frontrunning vulnerability caused by mutable on-chain order parameters, and a straightforward mitigation.

Two Anchor programs are included:

- frontrunning_vulnerable
- frontrunning_fixed

Both implement a simple maker/taker order flow using on-chain state.

---

## Vulnerability Overview

The vulnerable program allows a maker to modify the order price at any time.

When a taker submits a transaction to fill an order, the transaction:
- Does not specify the price it expects to trade at
- Blindly accepts the price stored on-chain at execution time

Because transactions are publicly visible in the mempool, a maker can observe a pending fill transaction and front-run it by updating the order price first.


---

## frontrunning_vulnerable

Behavior:
- Taker calls `fill_order(quantity)`
- Program reads `order.price` at execution time
- No constraint ensures the price matches what the taker intended

Attack flow:
1. Maker creates an order at a reasonable price
2. Taker submits a transaction to fill the order
3. Maker sees the transaction and submits `update_price` with a higher price
4. Taker’s transaction executes at the new price

Result:
- Taker overpays without violating any program checks

---

## frontrunning_fixed

Mitigation:
- Taker supplies `expected_price` when filling the order
- Program enforces `order.price == expected_price`

If the price has changed, the transaction fails.

This forces the order price to be effectively immutable from the taker’s perspective for the duration of the transaction.

---

## Security Takeaway

Any on-chain action that depends on mutable state must allow the caller to lock critical parameters.

Relying on “current on-chain values” without user-supplied constraints enables frontrunning.

---

## Notes

- Balance transfers are simplified and do not use real token programs
- This code is intentionally educational
- Not intended for production use

---
