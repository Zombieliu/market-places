use std::convert::TryInto;
use near_contract_standards::non_fungible_token::metadata::TokenMetadata;
use near_sdk::{AccountId};
use near_sdk_sim::{ContractAccount, STORAGE_AMOUNT, UserAccount, call, deploy, init_simulator, to_yocto,view};
use non_fungible_token::ContractContract;
use nft_marketplaces::{ContractContract as MarketContract, FungibleTokenId};
use near_sdk_sim::borsh::maybestd::collections::HashMap;
use near_sdk::json_types::U128;
use near_sdk::serde_json::json;


const NFT_CONTRACT_NAME: &str = "nft";
const MARKET_PLACES_NAME: &str = "marketplaces";


near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
   pub TOKEN_WASM_BYTES => "res/non_fungible_token.wasm",
   pub MARKET_PLACES_WASM_BYTES => "res/nft_marketplaces.wasm",
}

fn nft_init(
    initial_balance: u128,
) -> (UserAccount, ContractAccount<ContractContract>, UserAccount) {
    let mut genesis = near_sdk_sim::runtime::GenesisConfig::default();
    genesis.gas_price = 0;
    genesis.gas_limit = u64::MAX;
    let nft_master_account = init_simulator(Some(genesis));
    let nft_alice =
        nft_master_account.create_user(AccountId::new_unchecked("alice".to_string()), initial_balance);

    let nft_alice_account_id: AccountId = nft_alice.account_id().try_into().unwrap();

    let nft_contract_account = deploy! {
        contract: ContractContract,
        contract_id: NFT_CONTRACT_NAME,
        bytes: &TOKEN_WASM_BYTES,
        signer_account: nft_alice,
        init_method: new_default_meta(nft_alice_account_id)
    };

    (nft_master_account, nft_contract_account, nft_alice)
}

fn market_places_init(
    initial_balance: u128,
) -> (UserAccount, ContractAccount<MarketContract>, UserAccount) {
    let mut genesis = near_sdk_sim::runtime::GenesisConfig::default();
    genesis.gas_price = 0;
    genesis.gas_limit = u64::MAX;
    let market_master_account = init_simulator(Some(genesis));
    let market_bob =
        market_master_account.create_user(AccountId::new_unchecked("bob".to_string()), initial_balance);

    let bob_account_id: AccountId = market_bob.account_id().try_into().unwrap();

    let market_contract_account = deploy! {
        contract: MarketContract,
        contract_id: MARKET_PLACES_NAME,
        bytes: &MARKET_PLACES_WASM_BYTES,
        signer_account: market_bob,
        init_method: new(bob_account_id)
    };

    (market_master_account, market_contract_account, market_bob)
}

fn sample_token_metadata() -> TokenMetadata {
    TokenMetadata {
        title: Some("Olympus Mons".into()),
        description: Some("The tallest mountain in the charted solar system".into()),
        media: None,
        media_hash: None,
        copies: Some(1u64),
        issued_at: None,
        expires_at: None,
        starts_at: None,
        updated_at: None,
        extra: None,
        reference: None,
        reference_hash: None,
    }
}


fn market_approve_data() -> String {
    let mut near:HashMap<FungibleTokenId, U128> = HashMap::new();
    near.insert("near".parse().unwrap(),U128(100));
    json!({
        "sale_conditions": near,
        "is_auction": "",
        "is_royalties": "",
        "royalties": "",
    }).to_string()
}

#[test]
fn init_test() {
    let (_nft_master_account, _nft_contract_account, _nft_alice) = nft_init(to_yocto("500000"));
    let (_market_master_account, _market_contract_account, _market_bob) = market_places_init(to_yocto("500000"));
}


#[test]
fn check_promise() {
    let (_nft_master_account, nft_contract, nft_alice) = nft_init(to_yocto("500000"));
    let (_market_master_account, market_contract_account, market_bob) = market_places_init(to_yocto("500000"));
    let token_id = "P001".to_string();
    //alice.near
    let token_owner_id = nft_alice.account_id().try_into().unwrap();
    // get metadata
    let token_metadata = sample_token_metadata();
    // mint
    let res1 =call!(
        nft_alice,
        nft_contract.nft_mint(token_id, token_owner_id,token_metadata),
        deposit = STORAGE_AMOUNT
    );
    // let promise_outcomes = res1.get_receipt_results();
    println!("{:#?}", res1);

    // storage_deposit
    let deposit_account:AccountId = nft_alice.account_id().try_into().unwrap();
    let res2 =call!(
        market_bob,
        market_contract_account.storage_deposit(Some(deposit_account)),
        deposit = STORAGE_AMOUNT
    );
    println!("{:#?}", res2);

    //near
    let ft_token = view!(market_contract_account.supported_ft_token_ids());
    println!("{:#?}",ft_token.unwrap_json_value());

    // test storage_balance
    let alice_account = nft_alice.account_id().try_into().unwrap();
    let storage_balance_of = view!(market_contract_account.storage_balance_of(alice_account));
    println!("{:#?}",storage_balance_of.unwrap_json_value());

    // println!("{:#?}",market_contract_account.contract)

    //test nft_approve to market_places
    let msg = Some(market_approve_data());
    let token_id2 = "P001".to_string();
    let account_id:AccountId = market_contract_account.account_id().try_into().unwrap();
    // println!("{:#?}", account_id);
    let res2 =call!(
        nft_alice,
        nft_contract.nft_approve(
            token_id2,
            account_id.into(),
            msg
        ),
        deposit = STORAGE_AMOUNT

    );
    println!("{:#?}", res2);
    // println!("{:#?}",res2.unwrap_json_value());

}
