use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar}
};

use crate::{instructions::VaultInstruction, state::{UserAccount, VaultAccount}};
use crate::errors::VaultError;
use borsh::{BorshSerialize, BorshDeserialize};

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = VaultInstruction::unpack(instruction_data)?;

    match instruction {

        VaultInstruction::InitUserAccount {} => {
            msg!("User account created");
            process_init_user_account(program_id, accounts)
        }

        VaultInstruction::InitVaultAccount { data } => {
            msg!("Vaul Account Creation");
            process_init_vault(program_id, accounts, data)
        },

        VaultInstruction::EditVaultAccount { data } => {
            msg!("Vaul Account Creation");
            process_edit_vault(program_id, accounts, data)
        },

    }
}

fn process_init_user_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let user_wallet = next_account_info(accounts_iter)?;
    let system_program_info = next_account_info(accounts_iter)?;
    let user_account_info = next_account_info(accounts_iter)?;

    if !user_wallet.is_signer {
        msg!("Sorry, you are not the signer");
        return Err(VaultError::InvalidAccountData.into());
    }

    let (user_pda, vault_bump) = Pubkey::find_program_address(&[b"user", user_wallet.key.as_ref()], program_id);

    if user_pda != *user_account_info.key {
        return Err(VaultError::InvalidAccountData.into());
    }
    
    if user_account_info.data_is_empty() {
        
        let vaults: Vec<Pubkey> = Vec::new();
        let rent = Rent::get()?;
        let user_size = 32 + 4 + vaults.len()*32;

        invoke_signed(&system_instruction::create_account(
            user_wallet.key,
            user_account_info.key,
            rent.minimum_balance(user_size),
            user_size as u64,
            program_id
        ),
        &[
            user_wallet.clone(),
            user_account_info.clone(),
            system_program_info.clone(),
            ],
            &[&[b"user", user_wallet.key.as_ref(), &[vault_bump]]]
        )?;


        let user_data = UserAccount {
            user_address: *user_wallet.key,
            vaults
        };
        user_data.serialize(&mut &mut user_account_info.data.borrow_mut()[..])?;

    } else {
        msg!("User account already created!")
    }
    Ok(())
}
fn process_init_vault(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: Vec<u8>,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let user_wallet = next_account_info(accounts_iter)?;
    let user_account_info = next_account_info(accounts_iter)?;
    let system_program_info = next_account_info(accounts_iter)?;
    let vault_account_info = next_account_info(accounts_iter)?;

    if !user_wallet.is_signer {
        msg!("Sorry, you are not the signer");
        return Err(VaultError::InvalidAccountData.into());
    }

    let (vault_pda, vault_bump) = Pubkey::find_program_address(&[b"vault", user_wallet.key.as_ref()], program_id);
    let (user_pda, _user_bump) = Pubkey::find_program_address(&[b"user" ,user_wallet.key.as_ref()], program_id);


    if vault_pda != *vault_account_info.key ||
    user_pda != *user_account_info.key
    {
        return Err(VaultError::InvalidAccountData.into());
    }
    
    if vault_account_info.data_is_empty() {

        let rent = Rent::get()?;
        let vault_size = 32 + 4 + data.len();

        invoke_signed(&system_instruction::create_account(
            user_wallet.key,
            vault_account_info.key,
            rent.minimum_balance(vault_size),
            vault_size as u64,
            program_id
        ),
        &[
            user_wallet.clone(),
            vault_account_info.clone(),
            system_program_info.clone(),
            ],
            &[&[b"vault", user_wallet.key.as_ref(), &[vault_bump]]]
        )?;

        let vault_data = VaultAccount {
            user_account: *user_account_info.key,
            data,
        };
        vault_data.serialize(&mut &mut vault_account_info.data.borrow_mut()[..])?;

        let mut user_data = UserAccount::try_from_slice(&user_account_info.data.borrow())?;
        user_data.vaults.push(*vault_account_info.key);
        let new_size = user_account_info.data_len() +32;

        user_account_info.realloc(new_size, false)?;
        user_data.serialize(&mut &mut user_account_info.data.borrow_mut()[..])?;
        

    } else {
        msg!("Vault account already created!")
    }
    Ok(())
}

pub fn process_edit_vault(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: Vec<u8>,
) -> ProgramResult {
    let accounts_iter = & mut accounts.iter();
    let user_wallet = next_account_info(accounts_iter)?;
    // let user_account_info = next_account_info(accounts_iter)?;
    let vault_account_info = next_account_info(accounts_iter)?;
    let system_program_info = next_account_info(accounts_iter)?;

    if !user_wallet.is_signer {
        return Err(VaultError::InvalidAccountData.into());
    }

    // let (user_pda, _user_bump) = Pubkey::find_program_address(&[b"user" ,user_wallet.key.as_ref()], program_id);
    let (vault_pda, _vault_bump) = Pubkey::find_program_address(&[b"vault", user_wallet.key.as_ref()], program_id);

    if *vault_account_info.key != vault_pda
        {
        return Err(VaultError::InvalidAccountData.into());
    }

    let mut vault_data = VaultAccount::try_from_slice(&vault_account_info.data.borrow())?;

    let old_vault_size = vault_data.data.len();
    let new_size = data.len();
    let rent = Rent::get()?;

    if old_vault_size != new_size {
        let diff = if new_size > old_vault_size {
            new_size - old_vault_size
        } else {
            old_vault_size - new_size
        };

        let compensation = rent.minimum_balance(diff);

        if new_size > old_vault_size {
            invoke(
                &system_instruction::transfer(
                    user_wallet.key,
                    vault_account_info.key,
                    compensation,
                ),
                &[
                    user_wallet.clone(),
                    vault_account_info.clone(),
                    system_program_info.clone()
                    ],
            )?;
        } else {
            invoke(
                &system_instruction::transfer(
                    vault_account_info.key,
                    user_wallet.key,
                    compensation,
                ),
                &[
                    vault_account_info.clone(),
                    user_wallet.clone(),
                    system_program_info.clone()
                    ],
            )?;
        }

        vault_data.data = data;
        vault_account_info.realloc(new_size, false)?;
        vault_data.serialize(&mut &mut vault_account_info.data.borrow_mut()[..])?;

    } else {
        msg!("No changes are made.");
    }
    
    Ok(())
}