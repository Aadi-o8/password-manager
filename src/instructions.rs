use solana_program::{
    msg, program_error::ProgramError
};
use crate::errors::VaultError;
use borsh::{BorshSerialize, BorshDeserialize};

const MAX_BLOB_SIZE: usize = 1024;
const BYTE_SIZE_4: usize = 4;


#[derive(BorshSerialize, BorshDeserialize)]
pub enum VaultInstruction {

    InitUserAccount {},
    
    InitVaultAccount {
        data: Vec<u8>,
    },

    EditVaultAccount {
        data: Vec<u8>,
        index: u32,
        delete: u8,
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
                // let (&index,rest) = rest.split_first().ok_or(VaultError::DataUnpackError)?;
                let (&delete,rest) = rest.split_first().ok_or(VaultError::DataUnpackError)?;
                let (index,rest) = unpack_index(rest)?;
                let data = unpack_data(&rest)?;

                Self::EditVaultAccount {
                    data,
                    index,
                    delete,
                }            
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

pub fn unpack_index(input: &[u8]) -> Result<(u32, Vec<u8>), ProgramError> {

    if input.len() < BYTE_SIZE_4 {
        return Err(VaultError::DataUnpackError.into());
    }

    let (index, rest) = input.split_at(BYTE_SIZE_4);
    let index = u32::from_le_bytes(index.try_into().expect("Invalid Index"));

    Ok((index,rest.to_vec()))
}