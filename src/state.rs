use solana_program::pubkey::Pubkey;
use borsh::{BorshSerialize, BorshDeserialize};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct VaultAccount {
    pub name: [u8; 32],
    pub user_account: Pubkey,
    pub data: Vec<Credentials>,
}
#[derive(BorshSerialize, BorshDeserialize)]
pub struct UserAccount {
    pub user_address: Pubkey,
    pub vaults: Vec<Pubkey>,
}

#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct Credentials {
    field: [u8; 32],
    passkey: [u8; 32],
}