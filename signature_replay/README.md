# Signature Replay Vulnerability (secp256k1)

This repository demonstrates a common signature verification vulnerability on Solana: **signature replay** in secp256k1-based authorization flows.

The example intentionally ignores signature malleability and focuses purely on replay.

---

## Summary

The program verifies a valid secp256k1 signature and authorizes an action based on it.

However, the signature is not enforced to be single-use. As a result, the same signature can be replayed multiple times to repeatedly perform an authorized operation.

---

## Root Cause

The protocol treats signature verification as a stateless check:

- If the signature is valid, the action is allowed
- No on-chain state records whether the signature has already been used

This means the program cannot distinguish between:
- a fresh authorization
- a previously used (replayed) authorization

On Solana, transactions are replayable by design unless explicitly prevented.

---

## Vulnerable Pattern

A typical vulnerable flow:

1. User signs a message off-chain
2. Program verifies the signature on-chain
3. Program executes a privileged action
4. No nonce, sequence number, or usage tracking is enforced

The same signature can be submitted again to repeat the action.

---

## Impact

- Unauthorized repeated execution of privileged logic
- Draining of protocol resources
- Bypassing intended “one-time” authorization guarantees

No cryptographic break is required. The signature is valid.

---

## Fix

The fix is to enforce **single-use authorization**.

This is done by:
- Including a nonce or sequence number in the signed message
- Storing the highest accepted nonce on-chain
- Rejecting signatures that reuse or regress the nonce

This converts the signature from a reusable proof into a one-time authorization.

---

## What This Repository Contains

- A vulnerable instruction that accepts replayable signatures
- A safe instruction that enforces nonce-based replay protection
- Minimal state tracking to demonstrate the fix


---

## Disclaimer

This code is for educational and auditing purposes only.
It is not intended for production use without proper security review.
