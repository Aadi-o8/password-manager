use solana_program::pubkey::Pubkey;
use borsh::{BorshSerialize, BorshDeserialize};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct VaultAccount {
    pub user_account: Pubkey,
    pub data: Vec<u8>,
}
#[derive(BorshSerialize, BorshDeserialize)]
pub struct UserAccount {
    pub user_address: Pubkey,
    pub vaults: Vec<Pubkey>,
}