# Missing Account Constraint Vulnerability (Anchor)

This project demonstrates an authorization vulnerability caused by missing account constraints in Anchor, alongside a fixed implementation that correctly enforces ownership.

The examples are intentionally minimal and educational.

---

## Programs Included

- account_constraint_vulnerable
- account_constraint_fixed

Both programs manage a vault PDA that stores:
- an owner public key
- a logical balance

---

## Vulnerability Summary

Anchor only enforces authorization and relationships that are explicitly declared in the #[derive(Accounts)] context.

If a required constraint is missing, Anchor will:
- deserialize the account successfully
- accept any signer provided
- execute the instruction without enforcing ownership

This can allow unauthorized users to mutate protected state.

---

## Vulnerable Program

### Withdraw Account Context
```
#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub vault: Account<'info, Vault>,
    pub caller: Signer<'info>,
}
```

Issue:
- caller can be any signer
- There is no constraint tying caller to vault.owner

### Withdraw Instruction

vault.balance = vault.balance.checked_sub(amount).unwrap();

There is no ownership check in either the account context or the instruction logic.

### Impact

Any arbitrary signer can:
- pass themselves as caller
- supply someone else’s vault PDA
- withdraw funds as long as the balance is sufficient

---

## Fixed Program

### Withdraw Account Context

```
#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut, has_one = owner)]
    pub vault: Account<'info, Vault>,
    pub owner: Signer<'info>,
}
```

The has_one = owner constraint enforces:
- vault.owner == owner.key()
- owner must be a signer

This validation happens before the instruction logic executes.

### Withdraw Instruction
```
vault.balance = vault.balance.checked_sub(amount).unwrap();
```

No explicit owner check is needed in the instruction because Anchor has already enforced it.

---

## Key Takeaway

If an account relationship is security-critical, it must be expressed as an Anchor account constraint.

Anchor will not infer intent or protect you automatically.

Missing constraints are a common source of authorization bugs in Solana programs.

---

## Notes

- Balance changes are logical only; no real lamports are transferred
- Code is intentionally simplified
- Not production-ready
