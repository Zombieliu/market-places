use crate::*;


#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize,Debug,Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Bid {
    pub owner_id: AccountId,
    pub price: U128,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize,Debug,Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Sale {
    pub owner_id: AccountId,
    pub approval_id: u64,
    pub nft_contract_id: String,
    pub token_id: String,
    pub sale_conditions: SaleConditions,
    pub bids: Bids,
    pub created_at: U64,
    pub is_auction: bool,
    // Royalties
    pub is_royalties:bool,
    pub royalties:Royalties
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PurchaseArgs {
    pub nft_contract_id: AccountId,
    pub token_id: TokenId,
}

#[near_bindgen]
impl Contract {
    // for add sale see: nft_callbacks.rs

    /// TODO remove without redirect to wallet? panic reverts
    #[payable]
    pub fn remove_sale(&mut self, nft_contract_id: AccountId, token_id: String,ft_token_id:AccountId) {
        assert_one_yocto();
        let contract_token_id = format!("{}{}{}",nft_contract_id,DELIMETER,token_id);
        log!("{:#?}",contract_token_id);
        let mut last_sale = self.get_sale(contract_token_id).unwrap();
        log!("{:#?}",last_sale);
        let bid = last_sale.bids.get(&ft_token_id);
        match bid{
            Some(bid) =>{
                let last_owner = bid.clone().owner_id;
                log!("{:#?}",last_owner);
                let last_price = bid.clone().price.0;
                log!("{:#?}",last_price);
                Promise::new(last_owner).transfer(last_price);
                last_sale.bids.remove(&ft_token_id).expect("No bids");
                let mut sale = self.internal_remove_sale(nft_contract_id.into(), token_id);
                let owner_id = env::predecessor_account_id();
                assert_eq!(owner_id, sale.owner_id, "Must be sale owner");
            }
            None =>{
                let mut sale = self.internal_remove_sale(nft_contract_id.into(), token_id);
                let owner_id = env::predecessor_account_id();
                assert_eq!(owner_id, sale.owner_id, "Must be sale owner");
            }
        }
    }

    #[payable]
    pub fn update_price(
        &mut self,
        nft_contract_id: AccountId,
        token_id: String,
        ft_token_id: AccountId,
        price: U128,
    ) {
        assert_one_yocto();
        let contract_id: AccountId = nft_contract_id;
        let contract_and_token_id = format!("{}{}{}", contract_id, DELIMETER, token_id);
        let mut sale = self.sales.get(&contract_and_token_id).expect("No sale");
        assert_eq!(
            env::predecessor_account_id(),
            sale.owner_id,
            "Must be sale owner"
        );
        let bid = sale.bids.get(&ft_token_id);
        match bid {
            Some(bid) => {
                assert!(price.0 > bid.price.0, "update price must more than current bid");
            }
            None => {}
        }
        if !self.ft_token_ids.contains(&ft_token_id) {
            env::panic_str(format!("Token {} not supported by this market", ft_token_id).as_ref());
        }
        sale.sale_conditions.insert(ft_token_id, price);
        self.sales.insert(&contract_and_token_id, &sale);
    }

    #[payable]
    pub fn offer(&mut self, nft_contract_id: AccountId, token_id: String) {
        let contract_id: AccountId = nft_contract_id.into();
        let contract_and_token_id = format!("{}{}{}", contract_id, DELIMETER, token_id);
        let mut sale = self.sales.get(&contract_and_token_id).expect("No sale");
        let buyer_id = env::predecessor_account_id();
        assert_ne!(sale.owner_id, buyer_id, "Cannot bid on your own sale.");
        let ft_token_id = "near".parse().unwrap();
        let price = sale
            .sale_conditions
            .get(&ft_token_id)
            .expect("Not for sale in NEAR")
            ;
        let deposit = env::attached_deposit();
        assert!(deposit > 0, "Attached deposit must be greater than 0");

        // log!("{} {:#?} {:#?}",sale.is_auction,U128(deposit),U128(price.clone().0 * NEAR));
        if !sale.is_auction && U128(deposit) == U128(price.clone().0 * NEAR){
            self.process_purchase(
                contract_id,
                token_id,
                ft_token_id.clone(),
                U128(deposit),
                buyer_id.clone(),
            );
        }
       if !sale.is_auction && U128(deposit).0 < U128(price.clone().0 * NEAR).0{
            self.add_bid(
                contract_and_token_id,
                deposit,
                ft_token_id.clone(),
                buyer_id.clone(),
                &mut sale,
            );
        }

    }

    #[private]
    pub fn add_bid(
        &mut self,
        contract_and_token_id: ContractAndTokenId,
        amount: Balance,
        ft_token_id: AccountId,
        buyer_id: AccountId,
        sale: &mut Sale,
    ) {
        // store a bid and refund any current bid lower
        let new_bid = Bid {
            owner_id: buyer_id,
            price: U128(amount),
        };

        //get last bid
        let bid = sale.bids.get(&ft_token_id);
        match bid {
            Some(bid) => {
                if ft_token_id == AccountId::from("near".parse().unwrap()) {
                let last_owner = bid.clone().owner_id;
                let last_price = bid.clone().price.0;
                Promise::new(last_owner).transfer(last_price);
                sale.bids.insert(ft_token_id,new_bid.clone());
                self.sales.insert(&contract_and_token_id, &sale);
            } },
            None => {
                if ft_token_id == AccountId::from("near".parse().unwrap()) {
                    sale.bids.insert(ft_token_id,new_bid.clone());
                    self.sales.insert(&contract_and_token_id, &sale);
                    log!("{:#?}",&sale);
                }
            }
        }
    }

    pub fn accept_offer(
        &mut self,
        nft_contract_id: AccountId,
        token_id: String,
        ft_token_id: AccountId,
    ) {
        let contract_id: AccountId = nft_contract_id.into();
        let contract_and_token_id = format!("{}{}{}", contract_id.clone(), DELIMETER, token_id.clone());
        // remove bid before proceeding to process purchase
        let mut sale = self.sales.get(&contract_and_token_id).expect("No sale");
        let bids_for_token_id = sale.bids.remove(&ft_token_id).expect("No bids");
        let bid = &bids_for_token_id;
        self.sales.insert(&contract_and_token_id, &sale);
        // panics at `self.internal_remove_sale` and reverts above if predecessor is not sale.owner_id
        self.process_purchase(
            contract_id,
            token_id,
            ft_token_id.into(),
            bid.price,
            bid.owner_id.clone(),
        );
    }

    #[private]
    pub fn process_purchase(
        &mut self,
        nft_contract_id: AccountId,
        token_id: String,
        ft_token_id: AccountId,
        price: U128,
        buyer_id: AccountId,
    ) -> Promise {
        let sale = self.internal_remove_sale(nft_contract_id.clone(), token_id.clone());

        ext_contract::nft_transfer(
            buyer_id.clone(),
            token_id,
            Some(sale.approval_id),
            None,
            nft_contract_id,
            1,
            GAS_FOR_NFT_TRANSFER,
        )
        .then(ext_self::resolve_purchase(
            ft_token_id,
            buyer_id,
            sale,
            price,
            env::current_account_id(),
            NO_DEPOSIT,
            GAS_FOR_ROYALTIES,
        ))
    }



    #[private]
    pub fn resolve_purchase(
        &mut self,
        ft_token_id: AccountId,
        buyer_id: AccountId,
        sale: Sale,
        price: U128,
    ) -> U128 {

        // checking for payout information
        let tx_state = promise_result_as_success();
        log!("{:#?} {:#?} {:#?} {:#?} {:#?}",ft_token_id,buyer_id,price,tx_state,sale);
        let service_fee = service_fee_calculate(price);
        let seller = sale.owner_id;
        let amount:Balance = price.0 - service_fee;
        log!("{:#?} {:#?}",amount,service_fee);
        Promise::new(seller).transfer(amount);
        U128(1)
    }
}


    fn service_fee_calculate(price:U128) -> u128{
        return (price.0 *25)/1000
    }


#[ext_contract(ext_self)]
trait ExtSelf {
    fn resolve_purchase(
        &mut self,
        ft_token_id: AccountId,
        buyer_id: AccountId,
        sale: Sale,
        price: U128,
    ) -> Promise;
}
