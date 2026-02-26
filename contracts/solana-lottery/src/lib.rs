use anchor_lang::prelude::*;

declare_id!("LottrY111111111111111111111111111111111111");

#[program]
pub mod solana_lottery {
    use super::*;

    pub fn initialize_platform(ctx: Context<InitializePlatform>, fee_bps: u16) -> Result<()> {
        require!(fee_bps <= 1_000, LotteryError::InvalidFee); // 10% max for MVP

        let platform = &mut ctx.accounts.platform;
        platform.authority = ctx.accounts.authority.key();
        platform.fee_bps = fee_bps;
        platform.bump = ctx.bumps.platform;
        Ok(())
    }

    pub fn create_game(
        ctx: Context<CreateGame>,
        ticket_price_lamports: u64,
        max_tickets: u32,
        draw_slot: u64,
    ) -> Result<()> {
        require!(ticket_price_lamports > 0, LotteryError::InvalidTicketPrice);
        require!(max_tickets > 1, LotteryError::InvalidMaxTickets);
        require!(draw_slot > Clock::get()?.slot, LotteryError::InvalidDrawSlot);

        let game = &mut ctx.accounts.game;
        game.platform = ctx.accounts.platform.key();
        game.creator = ctx.accounts.authority.key();
        game.ticket_price_lamports = ticket_price_lamports;
        game.max_tickets = max_tickets;
        game.draw_slot = draw_slot;
        game.total_tickets = 0;
        game.winning_ticket = None;
        game.is_closed = false;
        game.bump = ctx.bumps.game;
        Ok(())
    }

    pub fn buy_ticket(ctx: Context<BuyTicket>) -> Result<()> {
        let game = &mut ctx.accounts.game;
        require!(!game.is_closed, LotteryError::GameClosed);
        require!(Clock::get()?.slot < game.draw_slot, LotteryError::DrawAlreadyStarted);
        require!(game.total_tickets < game.max_tickets, LotteryError::SoldOut);

        anchor_lang::system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: ctx.accounts.buyer.to_account_info(),
                    to: game.to_account_info(),
                },
            ),
            game.ticket_price_lamports,
        )?;

        let ticket = &mut ctx.accounts.ticket;
        ticket.game = game.key();
        ticket.owner = ctx.accounts.buyer.key();
        ticket.index = game.total_tickets;
        ticket.bump = ctx.bumps.ticket;

        game.total_tickets = game
            .total_tickets
            .checked_add(1)
            .ok_or(LotteryError::MathOverflow)?;

        Ok(())
    }

    pub fn close_game(ctx: Context<CloseGame>, winning_ticket: u32) -> Result<()> {
        let game = &mut ctx.accounts.game;
        require!(!game.is_closed, LotteryError::GameClosed);
        require!(Clock::get()?.slot >= game.draw_slot, LotteryError::DrawNotStarted);
        require!(game.total_tickets > 0, LotteryError::NoTicketsSold);
        require!(winning_ticket < game.total_tickets, LotteryError::InvalidWinningTicket);

        game.winning_ticket = Some(winning_ticket);
        game.is_closed = true;
        Ok(())
    }

    pub fn claim_prize(ctx: Context<ClaimPrize>) -> Result<()> {
        let game = &mut ctx.accounts.game;
        let platform = &ctx.accounts.platform;

        require!(game.is_closed, LotteryError::GameNotClosed);
        require!(game.winning_ticket == Some(ctx.accounts.ticket.index), LotteryError::NotWinner);
        require!(!ctx.accounts.ticket.claimed, LotteryError::AlreadyClaimed);

        let total_pot = game
            .ticket_price_lamports
            .checked_mul(game.total_tickets as u64)
            .ok_or(LotteryError::MathOverflow)?;
        let fee = total_pot
            .checked_mul(platform.fee_bps as u64)
            .ok_or(LotteryError::MathOverflow)?
            / 10_000;
        let winner_amount = total_pot.checked_sub(fee).ok_or(LotteryError::MathOverflow)?;

        **game.to_account_info().try_borrow_mut_lamports()? = game
            .to_account_info()
            .lamports()
            .checked_sub(winner_amount)
            .ok_or(LotteryError::MathOverflow)?;
        **ctx.accounts.winner.to_account_info().try_borrow_mut_lamports()? = ctx
            .accounts
            .winner
            .to_account_info()
            .lamports()
            .checked_add(winner_amount)
            .ok_or(LotteryError::MathOverflow)?;

        let ticket = &mut ctx.accounts.ticket;
        ticket.claimed = true;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializePlatform<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + PlatformConfig::INIT_SPACE,
        seeds = [b"platform", authority.key().as_ref()],
        bump
    )]
    pub platform: Account<'info, PlatformConfig>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateGame<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"platform", authority.key().as_ref()],
        bump = platform.bump,
        has_one = authority
    )]
    pub platform: Account<'info, PlatformConfig>,

    #[account(
        init,
        payer = authority,
        space = 8 + LotteryGame::INIT_SPACE,
        seeds = [b"game", platform.key().as_ref(), &draw_slot.to_le_bytes()],
        bump
    )]
    pub game: Account<'info, LotteryGame>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct BuyTicket<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(mut)]
    pub game: Account<'info, LotteryGame>,

    #[account(
        init,
        payer = buyer,
        space = 8 + TicketEntry::INIT_SPACE,
        seeds = [b"ticket", game.key().as_ref(), buyer.key().as_ref(), &game.total_tickets.to_le_bytes()],
        bump
    )]
    pub ticket: Account<'info, TicketEntry>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CloseGame<'info> {
    pub authority: Signer<'info>,

    #[account(
        seeds = [b"platform", authority.key().as_ref()],
        bump = platform.bump,
        has_one = authority
    )]
    pub platform: Account<'info, PlatformConfig>,

    #[account(mut, has_one = platform)]
    pub game: Account<'info, LotteryGame>,
}

#[derive(Accounts)]
pub struct ClaimPrize<'info> {
    #[account(mut)]
    pub winner: Signer<'info>,

    pub platform: Account<'info, PlatformConfig>,

    #[account(mut, has_one = platform)]
    pub game: Account<'info, LotteryGame>,

    #[account(
        mut,
        has_one = game,
        has_one = owner @ LotteryError::NotWinnerWallet,
        constraint = ticket.owner == winner.key() @ LotteryError::NotWinnerWallet
    )]
    pub ticket: Account<'info, TicketEntry>,

    /// CHECK: validated by has_one + constraint above
    pub owner: UncheckedAccount<'info>,
}

#[account]
#[derive(InitSpace)]
pub struct PlatformConfig {
    pub authority: Pubkey,
    pub fee_bps: u16,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct LotteryGame {
    pub platform: Pubkey,
    pub creator: Pubkey,
    pub ticket_price_lamports: u64,
    pub max_tickets: u32,
    pub total_tickets: u32,
    pub draw_slot: u64,
    pub winning_ticket: Option<u32>,
    pub is_closed: bool,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct TicketEntry {
    pub game: Pubkey,
    pub owner: Pubkey,
    pub index: u32,
    pub claimed: bool,
    pub bump: u8,
}

#[error_code]
pub enum LotteryError {
    #[msg("Invalid fee basis points")]
    InvalidFee,
    #[msg("Invalid ticket price")]
    InvalidTicketPrice,
    #[msg("Invalid max tickets")]
    InvalidMaxTickets,
    #[msg("Invalid draw slot")]
    InvalidDrawSlot,
    #[msg("Game is closed")]
    GameClosed,
    #[msg("Draw already started")]
    DrawAlreadyStarted,
    #[msg("Draw not started yet")]
    DrawNotStarted,
    #[msg("Game sold out")]
    SoldOut,
    #[msg("No tickets sold")]
    NoTicketsSold,
    #[msg("Invalid winning ticket")]
    InvalidWinningTicket,
    #[msg("Game not closed")]
    GameNotClosed,
    #[msg("Ticket is not the winner")]
    NotWinner,
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Prize already claimed")]
    AlreadyClaimed,
    #[msg("Winner wallet mismatch")]
    NotWinnerWallet,
}
