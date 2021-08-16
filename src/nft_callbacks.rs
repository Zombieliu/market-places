use crate::*;



/// approval callbacks from NFT Contracts

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SaleArgs {
    //FT id and number
    pub sale_conditions: SaleConditions,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_auction: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_royalties: Option<bool>,
    pub royalties: Royalties
}

pub trait NonFungibleTokenApprovalsReceiver {
    fn nft_on_approve(
        &mut self,
        token_id: TokenId,
        owner_id: AccountId,
        approval_id: u64,
        msg: String,
    );
}

#[near_bindgen]
impl NonFungibleTokenApprovalsReceiver for Contract {
    /// where we add the sale because we know nft owner can only call nft_approve

    fn nft_on_approve(
        &mut self,
        token_id: TokenId,
        owner_id: AccountId,
        approval_id: u64,
        msg: String,
    ) {
        //get user account
        let nft_contract_id = env::predecessor_account_id();
        let signer_id = env::signer_account_id();
        assert_ne!(
            nft_contract_id,
            signer_id,
            "nft_on_approve should only be called via cross-contract call"
        );
        assert_eq!(
            &owner_id,
            &signer_id,
            "owner_id should be signer_id"
        );
        // enforce signer's storage is enough to cover + 1 more sale

        let storage_amount = self.storage_amount().0;
        let owner_paid_storage = self.storage_deposits.get(&signer_id).unwrap_or(0);
        let signer_storage_required = (self.get_supply_by_owner_id(signer_id).0 + 1) as u128 * storage_amount;
        assert!(
            owner_paid_storage >= signer_storage_required,
            "Insufficient storage paid: {}, for {} sales at {} rate of per sale",
            owner_paid_storage, signer_storage_required / STORAGE_PER_SALE, STORAGE_PER_SALE
        );

        let SaleArgs { sale_conditions, is_auction,is_royalties,royalties } =
            near_sdk::serde_json::from_str(&msg).expect("Not valid SaleArgs");


        for (ft_token_id, _price) in sale_conditions.clone() {
            if !self.ft_token_ids.contains(&ft_token_id) {
                env::panic_str(
                    &format!("Token {} not supported by this market", ft_token_id),
                );
            }
        }

        // env::log(format!("add_sale for owner: {}", &owner_id).as_bytes());

        let bids = HashMap::new();
        //env::predecessor_account_id + || + TokenID
        let contract_and_token_id = format!("{}{}{}", nft_contract_id, DELIMETER, token_id);
        // Insert data
        self.sales.insert(
            &contract_and_token_id,
            &Sale {
                    owner_id: owner_id.clone().into(),
                    approval_id,
                    nft_contract_id: nft_contract_id.clone().into(),
                    token_id: token_id.clone(),
                    sale_conditions,
                    bids,
                    created_at: U64(env::block_timestamp()/1000000),
                    is_auction: is_auction.unwrap_or(false),
                    is_royalties:is_royalties.unwrap_or(false),
                    royalties: Option::from(royalties.unwrap_or(Default::default())),
            },
       );
       log!("contract_and_token_id: {}", &contract_and_token_id);



        // extra for views

        let mut by_owner_id = self.by_owner_id.get(&owner_id).unwrap_or_else(|| {
            UnorderedSet::new(
                StorageKey::ByOwnerIdInner {
                    account_id_hash: hash_account_id(&owner_id),
                }
                .try_to_vec()
                .unwrap(),
            )
        });

        let owner_occupied_storage = u128::from(by_owner_id.len()) * STORAGE_PER_SALE;
        assert!(
            owner_paid_storage > owner_occupied_storage,
            "User has more sales than storage paid"
        );
        // Insert data
        by_owner_id.insert(&contract_and_token_id);
        self.by_owner_id.insert(&owner_id, &by_owner_id);

        let mut by_nft_contract_id = self
            .by_nft_contract_id
            .get(&nft_contract_id)
            .unwrap_or_else(|| {
                UnorderedSet::new(
                    StorageKey::ByNFTContractIdInner {
                        account_id_hash: hash_account_id(&nft_contract_id),
                    }
                    .try_to_vec()
                    .unwrap(),
                )
            });
        // Insert data
        by_nft_contract_id.insert(&token_id);
        self.by_nft_contract_id
            .insert(&nft_contract_id, &by_nft_contract_id);
    }
}

