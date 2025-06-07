use solana_program::{
    msg, program_error::ProgramError
};
use crate::errors::VaultError;
use borsh::{BorshSerialize, BorshDeserialize};

const MAX_BLOB_SIZE: usize = 1024;

#[derive(BorshSerialize, BorshDeserialize)]
pub enum VaultInstruction {

    InitUserAccount {},
    
    InitVaultAccount {
        data: Vec<u8>,
    },

    EditVaultAccount {
        data: Vec<u8>,
    }
}

impl VaultInstruction {
    
    pub fn unpack(data: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = data.split_first().ok_or(VaultError::DataUnpackError)?;
        Ok(match tag {


            0 => {
                Self::InitUserAccount {  }
            }
            1 => {
                let data = unpack_data(rest)?;

                Self::InitVaultAccount { data }
            }
            2 => {
                let data = unpack_data(rest)?;

                Self::EditVaultAccount { data }            
            }
            _ => return Err(VaultError::InvalidInstruction.into())
        })
    }
}

pub fn unpack_data(input: &[u8]) -> Result<Vec<u8>, ProgramError> {

    if input.len() > MAX_BLOB_SIZE {
        msg!("Need to create new vault");
        return Err(VaultError::DataUnpackError.into());
    }
    Ok(input.to_vec())
}