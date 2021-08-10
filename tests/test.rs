use std::convert::TryInto;
use near_contract_standards::non_fungible_token::metadata::TokenMetadata;
use near_sdk::{AccountId, log};
use near_sdk_sim::{ContractAccount, DEFAULT_GAS, STORAGE_AMOUNT, UserAccount, call, deploy, init_simulator, to_yocto, view};


use non_fungible_token::ContractContract;

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
   pub TOKEN_WASM_BYTES => "res/non_fungible_token.wasm",
}

fn init(
    initial_balance: u128,
) -> (UserAccount, ContractAccount<ContractContract>, UserAccount) {
    let mut genesis = near_sdk_sim::runtime::GenesisConfig::default();
    genesis.gas_price = 0;
    genesis.gas_limit = u64::MAX;
    let master_account = init_simulator(Some(genesis));
    let alice =
        master_account.create_user(AccountId::new_unchecked("alice".to_string()), initial_balance);

    let alice_account_id: AccountId = alice.account_id().try_into().unwrap();

    let contract_account = deploy! {
        contract: ContractContract,
        contract_id: "contract",
        bytes: &TOKEN_WASM_BYTES,
        signer_account: master_account,
        init_method: new_default_meta(alice_account_id)
    };

    (master_account, contract_account, alice)
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

#[test]
fn init_test() {
    let (_master_account, _contract_account, _alice) = init(to_yocto("10000"));
}

#[test]
fn check_promise() {
    let (_master_account, contract, alice) = init(to_yocto("10000"));
    let token_id = "P001".to_string();
    let token_owner_id = alice.account_id().try_into().unwrap();
    let token_metadata = sample_token_metadata();
    let res =call!(
        alice,
        contract.nft_mint(token_id, token_owner_id,token_metadata),
        deposit = STORAGE_AMOUNT
    );
    let promise_outcomes = res.get_receipt_results();
    println!("{:#?}\n{:#?}", promise_outcomes, res);
    let res2 = view!(contract.nft_token("P001".to_string()));
    println!("{:#?}",res2.unwrap_json_value());

    // log!("{:#?}", res2);
}
