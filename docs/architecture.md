# Architecture: Lottery + Card Games + Sportsbook on Solana

## 1. Product Scope

### Phase 1 (MVP)
- Single lottery game type (fixed ticket price, fixed draw time).
- Ticket purchase and winner claim flow.
- Treasury fee split.

### Phase 2
- Multiple lottery templates (hourly/daily/jackpot).
- NFT/collectible lottery tickets.
- Referral and affiliate rewards.

### Phase 3
- Sportsbook markets (moneyline/spread/over-under) with oracle settlement.
- Provably fair card game modules (blackjack, poker variants).

## 2. On-chain Modules

### A) Lottery Program
Responsibilities:
- Create game accounts.
- Sell tickets.
- Lock game at draw time.
- Resolve winner (via randomness callback).
- Permit winner claim and treasury withdrawal.

Key accounts:
- `PlatformConfig`
- `LotteryGame`
- `TicketEntry`

### B) Sportsbook Program (future)
Responsibilities:
- Create events/markets.
- Accept and escrow bets.
- Settle from oracle outcome.
- Distribute winnings and fees.

### C) Card Games Program (future)
Responsibilities:
- Manage game rooms and stakes.
- Handle commit-reveal/VRF randomness.
- Enforce fair payout logic.

## 3. Off-chain Services

- **Indexer**: Reads on-chain events into Postgres.
- **Risk engine**: Exposure limits and anti-arbitrage checks.
- **Fraud monitoring**: Suspicious wallet behavior detection.
- **KYC/Geo service**: Regulatory gating by jurisdiction.
- **Admin console**: Market management, incident response.

## 4. Randomness & Oracles

Lottery and card games must use auditable randomness:
- Switchboard VRF or ORAO VRF for winner selection/card shuffling.

Sportsbook settlement requires reliable data feeds:
- Pyth or specialized sports data oracle adapter.

## 5. Security Requirements

- Program-derived-account ownership checks.
- Reentrancy-safe accounting patterns.
- Explicit overflow checks.
- Rate limits and pause/guardian role.
- Mandatory third-party audit before mainnet.

## 6. Compliance Notes

- Gambling products are regulated by jurisdiction.
- Integrate age checks, sanctions screening, and geofencing.
- Add AML monitoring and suspicious activity reporting.

## 7. Deployment Strategy

1. Devnet pilot (internal wallets only).
2. Closed beta (allowlist users).
3. Mainnet launch with conservative caps.
4. Gradual feature rollout per module.
