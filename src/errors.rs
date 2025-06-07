use solana_program::program_error::ProgramError;

#[derive(Debug)]
pub enum VaultError {
    InvalidAccountData,
    DataUnpackError,
    InvalidInstruction,
}

impl From<VaultError> for ProgramError {
    fn from(e: VaultError) -> Self { ProgramError::Custom(e as u32) }
}