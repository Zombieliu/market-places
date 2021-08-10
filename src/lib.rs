use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::collections::{UnorderedSet, LookupMap};
use near_sdk::{near_bindgen, AccountId, PanicOnDefault, BorshStorageKey, Balance, assert_one_yocto, Promise};
use near_sdk::env;
use near_sdk::env::STORAGE_PRICE_PER_BYTE;
use near_contract_standards::storage_management::StorageBalanceBounds;
use near_sdk::json_types::U128;
use near_contract_standards::non_fungible_token::TokenId;
use near_sdk::borsh::maybestd::collections::HashMap;
use near_sdk::log;


mod internal;
mod nft_callbacks;


/// per byes
const STORAGE_PER_SALE: u128 = 1000 * STORAGE_PRICE_PER_BYTE;

pub type FungibleTokenId = AccountId;
pub type ContractAndTokenId = String;
pub type SaleConditions = HashMap<FungibleTokenId, U128>;
pub type Amount = u8;
pub type Royalties = Option<HashMap<ContractAndTokenId,Amount>>;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub owner_id: AccountId,
    pub ft_token_ids: UnorderedSet<AccountId>,
    pub storage_deposits: LookupMap<AccountId, Balance>,
    pub by_owner_id: LookupMap<AccountId, UnorderedSet<ContractAndTokenId>>,
}

/// Helper structure to for keys of the persistent collections.
#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKey {
    FTTokenIds,
    StorageDeposits,
    ByOwnerId,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: AccountId){
       let mut this =  Self {
            owner_id,
            ft_token_ids: UnorderedSet::new(StorageKey::FTTokenIds),
            storage_deposits: LookupMap::new(StorageKey::StorageDeposits),
            by_owner_id: LookupMap::new(StorageKey::ByOwnerId),
       };
        // support NEAR by default
       this.ft_token_ids.insert(&AccountId::new_unchecked("near".to_string()));
    }

    /// only contract owner can add support FT token list
    /// and return success result
    pub fn add_ft_token_ids(&mut self, ft_token_ids: Vec<AccountId>) -> Vec<bool> {
        self.assert_owner();
        let mut added = vec![];
        for ft_token_id in ft_token_ids {
            added.push(self.ft_token_ids.insert(&ft_token_id));
        }
        added
    }

    /// TODO remove token (should check if sales can complete even if owner stops supporting token type)

    #[payable]
    pub fn storage_deposit(&mut self, account_id: Option<AccountId>) {
        let storage_account_id = account_id
            .map(|a| a.into())
            .unwrap_or_else(env::predecessor_account_id);

        let deposit = env::attached_deposit();
        // about 1kb 10TG
        assert!(
            deposit >= STORAGE_PER_SALE,
            "Requires minimum deposit of {}",
            STORAGE_PER_SALE
        );
        let mut balance: u128 = self.storage_deposits.get(&storage_account_id).unwrap_or(0);
        balance += deposit;
        self.storage_deposits.insert(&storage_account_id, &balance);
    }

    #[payable]
    pub fn storage_withdraw(&mut self) {
        assert_one_yocto();
        let owner_id = env::predecessor_account_id();
        let mut amount = self.storage_deposits.remove(&owner_id).unwrap_or(0);
        let sales = self.by_owner_id.get(&owner_id);
        let len = sales.map(|s| s.len()).unwrap_or_default();
        let diff = u128::from(len) * STORAGE_PER_SALE;
        amount -= diff;
        if amount > 0 {
            Promise::new(owner_id.clone()).transfer(amount);
        }
        if diff > 0 {
            self.storage_deposits.insert(&owner_id, &diff);
        }
    }

    /// views

    pub fn supported_ft_token_ids(&self) -> Vec<AccountId> {
        self.ft_token_ids.to_vec()
    }

    pub fn storage_balance_bounds(&self) -> StorageBalanceBounds {
        StorageBalanceBounds {
            min: U128(STORAGE_PER_SALE),
            max: None,
        }
    }

    pub fn storage_minimum_balance(&self) -> U128 {
        U128(STORAGE_PER_SALE)
    }

    pub fn storage_balance_of(&self, account_id: AccountId) -> U128 {
        U128(self.storage_deposits.get(&account_id).unwrap_or(0))
    }

    /// deprecated

    pub fn storage_paid(&self, account_id: AccountId) -> U128 {
        U128(self.storage_deposits.get(&account_id).unwrap_or(0))
    }

    pub fn storage_amount(&self) -> U128 {
        U128(STORAGE_PER_SALE)
    }
}
