use solana_program::{
    msg, program_error::ProgramError
};
use crate::errors::VaultError;
use borsh::{BorshSerialize, BorshDeserialize};

const MAX_BLOB_SIZE: usize = 64;
const BYTE_SIZE_4: usize = 4;
const NAME_SIZE_32:usize = 32;


#[derive(BorshSerialize, BorshDeserialize)]
pub enum VaultInstruction {

    InitUserAccount {},
    
    InitVaultAccount {
        vault_name: String,
        // data: Vec<u8>,
    },

    InitAddInVault {
        vault_name: String,
        data: [u8; 64],
    },

    EditVaultAccount {
        data: [u8; 64],
        vault_name:String,
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
                let vault_name = std::str::from_utf8(rest).map_err(|_| ProgramError::InvalidInstructionData)?.to_string();
                // let data = Self::unpack_data(&rest)?;

                Self::InitVaultAccount {
                    vault_name,
                    // data
                }
            }
            2 => {
                let (data,rest) = Self::unpack_data(&rest)?;
                let vault_name = std::str::from_utf8(rest).map_err(|_| ProgramError::InvalidInstructionData)?.to_string();
                Self::InitAddInVault {
                    vault_name,
                    data: Self::slice_to_array(data),
                }
            }
            3 => {
                // let (&index,rest) = rest.split_first().ok_or(VaultError::DataUnpackError)?;
                let (&delete,rest) = rest.split_first().ok_or(VaultError::DataUnpackError)?;
                let (index,rest) = Self::unpack_index(&rest)?;
                let (data,rest) = Self::unpack_data(&rest)?;
                let vault_name = std::str::from_utf8(rest).map_err(|_| ProgramError::InvalidInstructionData)?.to_string();

                Self::EditVaultAccount {
                    data: Self::slice_to_array(data),
                    index,
                    delete,
                    vault_name,
                }            
            }
            _ => return Err(VaultError::InvalidInstruction.into())
        })
    }
    

    pub fn unpack_data(input: &[u8]) -> Result< (&[u8], &[u8]), ProgramError > {
    
        if input.len() < MAX_BLOB_SIZE {
            // msg!("Need to create new vault");
            return Err(VaultError::DataUnpackError.into());
        }

        let (input_slice, rest) = input.split_at(MAX_BLOB_SIZE);

        Ok((input_slice,rest))
    }
    
    pub fn unpack_name(input: &[u8]) -> Result<(String, Vec<u8>), ProgramError> {
        if input.len() < NAME_SIZE_32 {
            msg!("Input too short for vault name");
            return Err(VaultError::DataUnpackError.into());
        }
        let (name, rest) = input.split_at(NAME_SIZE_32);
        let name = String::from_utf8(
            name.iter().cloned().take_while(|b| *b != 0).collect()
        ).map_err(|e| {
            msg!("Invalid UTF-8 in vault name: {}", e);
            VaultError::DataUnpackError
        })?;
        Ok((name, rest.to_vec()))
    }
    
    pub fn unpack_index(input: &[u8]) -> Result<(u32, Vec<u8>), ProgramError> {
    
        if input.len() < BYTE_SIZE_4 {
            return Err(VaultError::DataUnpackError.into());
        }
    
        let (index, rest) = input.split_at(BYTE_SIZE_4);
        let index = u32::from_le_bytes(index.try_into().expect("Invalid Index"));
    
        Ok((index,rest.to_vec()))
    }

    fn slice_to_array(slice: &[u8]) -> [u8; 64] {
    slice.try_into().expect("slice must be exactly 64 bytes")
}
}
