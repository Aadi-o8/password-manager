use std::usize;

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};

use crate::errors::VaultError;
use crate::{
    instructions::VaultInstruction,
    state::{Credentials, UserAccount, VaultAccount},
};
use borsh::{BorshDeserialize, BorshSerialize};

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

        VaultInstruction::InitVaultAccount {
            vault_name,
            // data
        } => {
            msg!("Vault Account creation");
            process_init_vault(program_id, accounts, vault_name)
        }

        VaultInstruction::InitAddInVault {
            vault_name,
            data
        } => {
            msg!("Adding in Vault account");
            process_add_in_vault(program_id, accounts, data, vault_name)
        }

        VaultInstruction::EditVaultAccount {
            data,
            vault_name,
            index,
            delete,
        } => {
            msg!("Vaul Account Creation");
            process_edit_vault(program_id, accounts, data, index, delete, vault_name)
        }
    }
}


fn process_init_user_account(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let user_wallet = next_account_info(accounts_iter)?;
    let user_account_info = next_account_info(accounts_iter)?;
    let system_program_info = next_account_info(accounts_iter)?;

    if !user_wallet.is_signer {
        msg!("Sorry, you are not the signer");
        return Err(VaultError::InvalidAccountData.into());
    }

    let (user_pda, user_bump) =
        Pubkey::find_program_address(&[b"user_at_password_manager", user_wallet.key.as_ref()], program_id);

    if user_pda != *user_account_info.key {
        return Err(VaultError::InvalidAccountData.into());
    }

    if user_account_info.data_is_empty() {
        let vaults: Vec<Pubkey> = Vec::new();
        let rent = Rent::get()?;
        let user_size = 32 + 4 ;

        invoke_signed(
            &system_instruction::create_account(
                user_wallet.key,
                user_account_info.key,
                rent.minimum_balance(user_size),
                user_size as u64,
                program_id,
            ),
            &[
                user_wallet.clone(),
                user_account_info.clone(),
                system_program_info.clone(),
            ],
            &[&[b"user_at_password_manager", user_wallet.key.as_ref(), &[user_bump]]],
        )?;

        let user_data = UserAccount {
            user_address: *user_wallet.key,
            vaults,
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
    vault_name: String,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let user_wallet = next_account_info(accounts_iter)?;
    let user_account_info = next_account_info(accounts_iter)?;
    let vault_account_info = next_account_info(accounts_iter)?;
    let system_program_info = next_account_info(accounts_iter)?;

    if !user_wallet.is_signer {
        msg!("Error: user_wallet is not signer");
        return Err(VaultError::InvalidAccountData.into());
    }

    let (vault_pda, vault_bump) = Pubkey::find_program_address(
        &[b"vault", user_wallet.key.as_ref(), vault_name.as_bytes()],
        program_id,
    );
    let (user_pda, _user_bump) = Pubkey::find_program_address(
        &[b"user_at_password_manager", user_wallet.key.as_ref()],
        program_id,
    );

    if vault_pda != *vault_account_info.key {
        msg!("Error: Invalid vault PDA. Expected {}, got {}", vault_pda, vault_account_info.key);
        return Err(VaultError::InvalidAccountData.into());
    }
    if user_pda != *user_account_info.key {
        msg!("Error: Invalid user PDA. Expected {}, got {}", user_pda, user_account_info.key);
        return Err(VaultError::InvalidAccountData.into());
    }

    // Verify user account is initialized and program-owned
    if user_account_info.data_is_empty() || *user_account_info.owner != *program_id {
        msg!("Error: User account not initialized or not owned by program");
        return Err(VaultError::InvalidAccountData.into());
    }

    // Validate user account data
    let mut user_data = UserAccount::try_from_slice(&user_account_info.data.borrow())?;

    msg!("Vault name: {}", vault_name);

    if vault_account_info.data_is_empty() {
        let rent = Rent::get()?;
        // Adjust vault_size based on VaultAccount struct
        let vault_size = 32 + 32 + 4; // Discriminator + Pubkey + String (len + data) + Vec (len)

        // Create vault account
        invoke_signed(
            &system_instruction::create_account(
                user_wallet.key,
                vault_account_info.key,
                rent.minimum_balance(vault_size),
                vault_size as u64,
                program_id,
            ),
            &[
                user_wallet.clone(),
                vault_account_info.clone(),
                system_program_info.clone(),
            ],
            &[&[b"vault", user_wallet.key.as_ref(), vault_name.as_bytes(), &[vault_bump]]],
        )?;

        // Calculate new user account size
        let old_size = user_account_info.data_len();
        let new_size = old_size + 32; // Add space for vault Pubkey
        let old_rent = rent.minimum_balance(old_size);
        let new_rent = rent.minimum_balance(new_size);
        let rent_diff = new_rent.saturating_sub(old_rent);

        if rent_diff > 0 {
            invoke(
                &system_instruction::transfer(user_wallet.key, user_account_info.key, rent_diff),
                &[
                    user_wallet.clone(),
                    user_account_info.clone(),
                    system_program_info.clone(),
                ],
            )?;
        }

        let data=Vec::new();
        // Converting the fund_name to an array of u8 of fixed size 32
        let bytes = vault_name.as_bytes();
        let mut array = [0u8; 32];
        let len = bytes.len().min(32);
        array[..len].copy_from_slice(&bytes[..len]);
        // Serialize vault data
        let vault_data = VaultAccount {
            name: array,
            user_account: *user_account_info.key,
            data,
        };
        vault_data.serialize(&mut &mut vault_account_info.data.borrow_mut()[..])?;

        // Update user account
        user_account_info.realloc(new_size, false)?;
        // let mut user_data = UserAccount {
        //     user_address: user_pda,
        //     vaults: user_data.vaults,
        // };
        user_data.vaults.push(*vault_account_info.key);
        user_data.serialize(&mut &mut user_account_info.data.borrow_mut()[..])?;

        msg!("Vault created: {} for user {}", vault_name, user_wallet.key);
    } else {
        msg!("Error: Vault account already exists");
        return Err(VaultError::InvalidAccountData.into());
    }
    Ok(())
}

pub fn process_add_in_vault(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data:[u8; 64] ,
    vault_name: String,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let user_wallet = next_account_info(accounts_iter)?;
    // let user_account_info = next_account_info(accounts_iter)?;
    let system_program_info = next_account_info(accounts_iter)?;
    let vault_account_info = next_account_info(accounts_iter)?;

    if !user_wallet.is_signer {
        msg!("Sorry, you are not the signer");
        return Err(VaultError::InvalidAccountData.into());
    }

    let (vault_pda, _vault_bump) =
        Pubkey::find_program_address(&[b"vault", user_wallet.key.as_ref(), vault_name.as_bytes()], program_id);

    if vault_pda != *vault_account_info.key {
        return Err(VaultError::InvalidAccountData.into());
    }

    if vault_account_info.data_len() > 10176 {
        msg!("Need to create a new vault");
        return Err(VaultError::TooMuchData.into());
    }

    // Checking if the vault account is empty
    if !vault_account_info.data_is_empty() {
        let credential = Credentials::try_from_slice(&data)?;
        let mut vault_data = VaultAccount::try_from_slice(&vault_account_info.data.borrow())?;

        let rent = Rent::get()?;
        let old_size = vault_data.data.len();
        let new_size = old_size + data.len();
        let new_rent = rent.minimum_balance(new_size);
        let old_rent = rent.minimum_balance(old_size);
        let rent_diff = new_rent - old_rent;

        // Paying rent for the added bytes in the vault account
        invoke(
            &system_instruction::transfer(
                user_wallet.key,
                vault_account_info.key,
                rent_diff
            ),
            &[
                user_wallet.clone(),
                vault_account_info.clone(),
                system_program_info.clone(),
            ],
        )?;

        // Reallocating the added bytes in the vault account
        vault_account_info.realloc(new_size, false)?;
        // Adding new data in the vault account(encrypted data)
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
    data: [u8; 64],
    index: u32,
    delete: u8,
    vault_name: String,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let user_wallet = next_account_info(accounts_iter)?;
    let vault_account_info = next_account_info(accounts_iter)?;
    let system_program_info = next_account_info(accounts_iter)?;

    if !user_wallet.is_signer {
        msg!("Sorry, you are not the signer");
        return Err(VaultError::InvalidAccountData.into());
    }

    let (vault_pda, _vault_bump) =
        Pubkey::find_program_address(&[b"vault", user_wallet.key.as_ref(), vault_name.as_bytes()], program_id);

    if *vault_account_info.key != vault_pda {
        return Err(VaultError::InvalidAccountData.into());
    }

    // Checking if user want to edit the data or delete it
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
        let rent_diff = old_rent - new_rent;

        // Paying the rent recovered to the user wallet
        invoke(
            &system_instruction::transfer(
                vault_account_info.key,
                user_wallet.key,
                rent_diff
            ),
            &[
                user_wallet.clone(),
                vault_account_info.clone(),
                system_program_info.clone(),
            ],
        )?;

        // Reallocating the size of data in the vault account
        vault_account_info.realloc(new_size, false)?;
        // Removing the chosen data from the vault account
        let mut vault_data = VaultAccount::try_from_slice(&vault_account_info.data.borrow())?;
        vault_data.data.remove(index as usize);
        vault_data.serialize(&mut &mut vault_account_info.data.borrow_mut()[..])?;
    }

    Ok(())
}
