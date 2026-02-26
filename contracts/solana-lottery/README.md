# solana-lottery (Anchor Program)

This is an MVP lottery contract with the following instructions:

- `initialize_platform(fee_bps)`
- `create_game(ticket_price_lamports, max_tickets, draw_slot)`
- `buy_ticket()`
- `close_game(winning_ticket)`
- `claim_prize()`

## Security + correctness notes

- Uses checked math for pot calculations.
- Restricts platform/game creation with PDA constraints.
- Defers fair randomness integration to a VRF callback flow.

## Build

```bash
cargo check
```

## Important production upgrades

- Replace admin-supplied `winning_ticket` with VRF.
- Add treasury-withdraw instruction for fees.
- Add game state machine with explicit statuses.
- Add integration tests on local validator.
