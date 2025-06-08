use std::usize;

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke_signed, invoke},
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar}
};

use crate::{instructions::VaultInstruction, state::{Credentials, UserAccount, VaultAccount}};
use crate::errors::VaultError;
use borsh::{BorshSerialize, BorshDeserialize};

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
    // index: u64,
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

        VaultInstruction::EditVaultAccount { data, index , delete} => {
            msg!("Vaul Account Creation");
            process_edit_vault(program_id, accounts, data, index, delete)
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

        let credential = Credentials::try_from_slice(&data[4..])?;
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

        let mut data = Vec::new();
        data.push(credential);

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
        msg!("Vault account already created!");
        return Err(VaultError::InvalidAccountData.into());
    }
    Ok(())
}

pub fn process_add_in_vault(
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

    let (vault_pda, _vault_bump) = Pubkey::find_program_address(&[b"vault", user_wallet.key.as_ref()], program_id);
    let (user_pda, _user_bump) = Pubkey::find_program_address(&[b"user" ,user_wallet.key.as_ref()], program_id);


    if vault_pda != *vault_account_info.key ||
    user_pda != *user_account_info.key
    {
        return Err(VaultError::InvalidAccountData.into());
    }

    if !vault_account_info.data_is_empty() {
        let credential = Credentials::try_from_slice(&data)?;
        let mut vault_data = VaultAccount::try_from_slice(&vault_account_info.data.borrow())?;
        
        let rent = Rent::get()?;
        let old_size = vault_data.data.len();
        let new_size = old_size + data.len();
        let new_rent = rent.minimum_balance(new_size);
        let old_rent = rent.minimum_balance(old_size);
        let rent_diff = new_rent-old_rent;


        invoke(
            &system_instruction::transfer(
                user_wallet.key,
                vault_account_info.key,
                rent_diff,
            ),
            &[
                user_wallet.clone(),
                vault_account_info.clone(),
                system_program_info.clone()
                ],
        )?;

        vault_account_info.realloc(new_size, false)?;
        vault_data.data.push(credential);
        vault_data.serialize(&mut &mut vault_account_info.data.borrow_mut()[..])?;
    } else {
        msg!("Vault account is not created yet");
        return Err(VaultError::InvalidAccountData.into());
    }

    Ok(())
}

pub fn process_edit_vault(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: Vec<u8>,
    index: u32,
    delete: u8,
) -> ProgramResult {
    let accounts_iter = & mut accounts.iter();
    let user_wallet = next_account_info(accounts_iter)?;
    // let user_account_info = next_account_info(accounts_iter)?;
    let vault_account_info = next_account_info(accounts_iter)?;
    let system_program_info = next_account_info(accounts_iter)?;

    if !user_wallet.is_signer {
        msg!("Sorry, you are not the signer");
        return Err(VaultError::InvalidAccountData.into());
    }

    // let (user_pda, _user_bump) = Pubkey::find_program_address(&[b"user" ,user_wallet.key.as_ref()], program_id);
    let (vault_pda, _vault_bump) = Pubkey::find_program_address(&[b"vault", user_wallet.key.as_ref()], program_id);
    
    if *vault_account_info.key != vault_pda
        {
            return Err(VaultError::InvalidAccountData.into());
    }

    if delete == 0 {    

        let mut vault_data = VaultAccount::try_from_slice(&vault_account_info.data.borrow())?;
        let credential = Credentials::try_from_slice(&data)?;
        vault_data.data[index as usize] = credential;
        vault_data.serialize(&mut &mut vault_account_info.data.borrow_mut()[..])?;

    } else {
        let rent = Rent::get()?;
        let old_size = vault_account_info.data_len();
        let new_size = old_size - data.len();
        let new_rent = rent.minimum_balance(new_size);
        let old_rent = rent.minimum_balance(old_size);
        let rent_diff = old_rent-new_rent;
        
        invoke(
            &system_instruction::transfer(
                vault_account_info.key,
                user_wallet.key,
                rent_diff,
            ),
            &[
                user_wallet.clone(),
                vault_account_info.clone(),
                system_program_info.clone()
                ],
            )?;
            
            vault_account_info.realloc(new_size, false)?;
            let mut vault_data = VaultAccount::try_from_slice(&vault_account_info.data.borrow())?;
            vault_data.data.remove(index as usize);
            vault_data.serialize(&mut &mut vault_account_info.data.borrow_mut()[..])?;
        }
    
    Ok(())
}