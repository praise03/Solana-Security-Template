#![no_std]

use pinocchio::nostd_panic_handler;
use pinocchio::{
    account_info::AccountInfo,
    entrypoint,
    program_error::ProgramError,
    pubkey::{find_program_address, Pubkey},
    ProgramResult,
};


entrypoint!(process_instruction);
nostd_panic_handler!();


pub const ID: Pubkey = [
    1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
    1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
];

/// ─────────────────────────────────────────
/// Instruction dispatch
/// ─────────────────────────────────────────
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    match instruction_data.split_first() {
        Some((UpdateFeeVulnerable::DISCRIMINATOR, data)) => {
            UpdateFeeVulnerable::try_from((program_id, accounts, data))?.process()
        }
        Some((UpdateFeeSafe::DISCRIMINATOR, data)) => {
            UpdateFeeSafe::try_from((program_id, accounts, data))?.process()
        }
        _ => Err(ProgramError::InvalidInstructionData),
    }
}

#[repr(C)]
pub struct ConfigState {
    pub admin: Pubkey,
    pub protocol_fee: u64,
}

pub struct UpdateFeeAccounts<'a> {
    pub config: &'a AccountInfo,
    pub admin: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for UpdateFeeAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [config, admin] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if config.data_len() != core::mem::size_of::<ConfigState>() {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self { config, admin })
    }
}


pub struct UpdateFeeData {
    pub new_fee: u64,
}

impl TryFrom<&[u8]> for UpdateFeeData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() != 8 {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(Self {
            new_fee: u64::from_le_bytes(data.try_into().map_err(|_| ProgramError::InvalidInstructionData)?),
        })
    }
}

pub struct UpdateFeeVulnerable<'a> {
    pub accounts: UpdateFeeAccounts<'a>,
    pub data: UpdateFeeData,
}

impl<'a> UpdateFeeVulnerable<'a> {
    pub const DISCRIMINATOR: &'a u8 = &0;

    pub fn process(&self) -> ProgramResult {
        let mut config_data = self.accounts.config.try_borrow_mut_data()?;
        let config = unsafe { &mut *(config_data.as_mut_ptr() as *mut ConfigState) };

        // ❌ VULNERABLE: only checks pubkey equality
        if config.admin != *self.accounts.admin.key() {
            return Err(ProgramError::InvalidAccountData);
        }

        //Missing admin.is_signer()

        config.protocol_fee = self.data.new_fee;


        Ok(())
    }
}

impl<'a> TryFrom<(&Pubkey, &'a [AccountInfo], &'a [u8])> for UpdateFeeVulnerable<'a> {
    type Error = ProgramError;

    fn try_from(
        (_program_id, accounts, data): (&Pubkey, &'a [AccountInfo], &'a [u8]),
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            accounts: UpdateFeeAccounts::try_from(accounts)?,
            data: UpdateFeeData::try_from(data)?,
        })
    }
}

pub struct UpdateFeeSafe<'a> {
    pub accounts: UpdateFeeAccounts<'a>,
    pub data: UpdateFeeData,
}

impl<'a> UpdateFeeSafe<'a> {
    pub const DISCRIMINATOR: &'a u8 = &1;

    pub fn process(&self) -> ProgramResult {
        let mut config_data = self.accounts.config.try_borrow_mut_data()?;
        let config = unsafe { &mut *(config_data.as_mut_ptr() as *mut ConfigState) };

        if config.admin != *self.accounts.admin.key() {
            return Err(ProgramError::InvalidAccountData);
        }

        // ✅ FIX: require signature
        if !self.accounts.admin.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        config.protocol_fee = self.data.new_fee;


        Ok(())
    }
}

impl<'a> TryFrom<(&Pubkey, &'a [AccountInfo], &'a [u8])> for UpdateFeeSafe<'a> {
    type Error = ProgramError;

    fn try_from(
        (_program_id, accounts, data): (&Pubkey, &'a [AccountInfo], &'a [u8]),
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            accounts: UpdateFeeAccounts::try_from(accounts)?,
            data: UpdateFeeData::try_from(data)?,
        })
    }
}