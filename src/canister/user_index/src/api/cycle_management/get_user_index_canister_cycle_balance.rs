use ic_cdk::api;

#[ic_cdk::query]
#[candid::candid_method(query)]
fn get_user_index_canister_cycle_balance() -> u128 {
    api::canister_balance128()
}
