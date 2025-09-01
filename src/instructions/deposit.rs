use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::find_program_address, ProgramResult};
use pinocchio_system::instructions::Transfer;

pub struct Deposit<'a>{
    pub accounts: DepositAccounts<'a>,
    pub  instruction_data: DepositInstructionData
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for Deposit<'a>   {
    type Error = ProgramError;

    fn try_from((data, accounts): (&'a [u8], &'a [AccountInfo])) -> Result<Self, Self::Error> {
       let accounts = DepositAccounts::try_from(accounts)?;
       let instruction_data = DepositInstructionData::try_from(data)?;

       Ok(Self {
        accounts,
        instruction_data
       }) 
    }
}

impl<'a>  Deposit<'a> {
  pub const DISCRIMINATOR: &'a u8  = &0;

  pub fn process(&mut self) -> ProgramResult {
    Transfer {
        from: self.accounts.owner,
        to: self.accounts.vault,
        lamports: self.instruction_data.amount
    }.invoke()
  }
}

pub struct  DepositAccounts<'a>{
    pub vault: &'a AccountInfo,
    pub owner: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for DepositAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [owner, vault] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Check if the Owner is the signer
        if !owner.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // check if the vault account is owned by the program
        if !vault.is_owned_by(&pinocchio_system::ID){
            return Err(ProgramError::InvalidAccountOwner);
        }

        // Check if the vault has lamports
        if vault.lamports().ne(&0) {
            return  Err(ProgramError::InvalidAccountData);
        }

        let (vault_key, _) = find_program_address(&[b"vault", owner.key()], &crate::ID);

        // check the vault keys are the same
        if vault.key().ne(&vault_key) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        Ok(Self { vault, owner})

    }
}

pub struct  DepositInstructionData{
    pub  amount: u64,
}

impl<'a> TryFrom<&'a [u8]> for DepositInstructionData {
    type Error = ProgramError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        // Check if the data length is equal to the amount
        if data.len() != size_of::<u64>(){
            return Err(ProgramError::InvalidInstructionData);
        }
        // Get amount 
        let amount = u64::from_le_bytes(data.try_into().unwrap());

        // Check if the amount is zero
        if amount.eq(&0){
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(Self {amount})
    }
}