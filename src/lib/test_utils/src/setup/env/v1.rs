use std::{collections::HashMap, env, path::Path};

use candid::Principal;
use ic_test_state_machine_client::{CanisterSettings, StateMachine};
use shared_utils::{
    access_control::UserAccessRole,
    canister_specific::{
        configuration::types::args::ConfigurationInitArgs,
        data_backup::types::args::DataBackupInitArgs, post_cache::types::arg::PostCacheInitArgs,
        user_index::types::args::UserIndexInitArgs,
    },
    common::types::known_principal::{KnownPrincipalMap, KnownPrincipalType},
};

use crate::setup::test_constants::{
    get_canister_wasm, get_global_super_admin_principal_id,
    v1::{
        CANISTER_INITIAL_CYCLES_FOR_NON_SPAWNING_CANISTERS,
        CANISTER_INITIAL_CYCLES_FOR_SPAWNING_CANISTERS,
    },
};

/// The path to the state machine binary to run the tests with
pub static STATE_MACHINE_BINARY: &str = "../../../ic-test-state-machine";

pub fn get_new_state_machine() -> StateMachine {
    let path = match env::var_os("DFX_IC_STATE_MACHINE_TESTS_PATH") {
        None => STATE_MACHINE_BINARY.to_string(),
        Some(path) => path
            .clone()
            .into_string()
            .unwrap_or_else(|_| panic!("Invalid string path for {path:?}")),
    };

    if !Path::new(&path).exists() {
        println!("
        Could not find state machine binary to run canister integration tests.

        I looked for it at {:?}. You can specify another path with the environment variable STATE_MACHINE_BINARY (note that I run from {:?}).

        Run the following command to get the binary:
            curl -sLO https://download.dfinity.systems/ic/a8da3aa23dc6f8f4708cb0cb8edce84c5bd8f225/binaries/x86_64-linux/ic-test-state-machine.gz
            gzip -d ic-test-state-machine.gz
            chmod +x ic-test-state-machine
        where $commit can be read from `.ic-commit` and $platform is 'x86_64-linux' for Linux and 'x86_64-darwin' for Intel/rosetta-enabled Darwin.
        ", &path, &env::current_dir().map(|x| x.display().to_string()).unwrap_or_else(|_| "an unknown directory".to_string()));
    }

    StateMachine::new(&path, false)
}

pub fn get_initialized_env_with_provisioned_known_canisters(
    state_machine: &StateMachine,
) -> KnownPrincipalMap {
    let canister_provisioner = |cycle_amount: u128| {
        let settings = Some(CanisterSettings {
            controllers: Some(vec![get_global_super_admin_principal_id()]),
            ..Default::default()
        });
        let canister_id = state_machine
            .create_canister_with_settings(settings, Some(get_global_super_admin_principal_id()));
        state_machine.add_cycles(canister_id, cycle_amount);
        canister_id
    };

    // * Provision canisters
    let mut known_principal_map_with_all_canisters = KnownPrincipalMap::default();
    known_principal_map_with_all_canisters.insert(
        KnownPrincipalType::UserIdGlobalSuperAdmin,
        get_global_super_admin_principal_id(),
    );
    known_principal_map_with_all_canisters.insert(
        KnownPrincipalType::CanisterIdConfiguration,
        canister_provisioner(CANISTER_INITIAL_CYCLES_FOR_NON_SPAWNING_CANISTERS),
    );
    known_principal_map_with_all_canisters.insert(
        KnownPrincipalType::CanisterIdDataBackup,
        canister_provisioner(CANISTER_INITIAL_CYCLES_FOR_NON_SPAWNING_CANISTERS),
    );
    known_principal_map_with_all_canisters.insert(
        KnownPrincipalType::CanisterIdPostCache,
        canister_provisioner(CANISTER_INITIAL_CYCLES_FOR_NON_SPAWNING_CANISTERS),
    );
    known_principal_map_with_all_canisters.insert(
        KnownPrincipalType::CanisterIdUserIndex,
        canister_provisioner(CANISTER_INITIAL_CYCLES_FOR_SPAWNING_CANISTERS),
    );

    // * Install canisters
    let canister_installer = |canister_id: Principal, wasm_module: Vec<u8>, arg: Vec<u8>| {
        state_machine.install_canister(
            canister_id,
            wasm_module,
            arg,
            Some(get_global_super_admin_principal_id()),
        );
    };

    canister_installer(
        *known_principal_map_with_all_canisters
            .get(&KnownPrincipalType::CanisterIdConfiguration)
            .unwrap(),
        get_canister_wasm(KnownPrincipalType::CanisterIdConfiguration),
        candid::encode_one(ConfigurationInitArgs {
            known_principal_ids: Some(known_principal_map_with_all_canisters.clone()),
            ..Default::default()
        })
        .unwrap(),
    );
    canister_installer(
        *known_principal_map_with_all_canisters
            .get(&KnownPrincipalType::CanisterIdDataBackup)
            .unwrap(),
        get_canister_wasm(KnownPrincipalType::CanisterIdDataBackup),
        candid::encode_one(DataBackupInitArgs {
            known_principal_ids: Some(known_principal_map_with_all_canisters.clone()),
            ..Default::default()
        })
        .unwrap(),
    );
    canister_installer(
        *known_principal_map_with_all_canisters
            .get(&KnownPrincipalType::CanisterIdPostCache)
            .unwrap(),
        get_canister_wasm(KnownPrincipalType::CanisterIdPostCache),
        candid::encode_one(PostCacheInitArgs {
            known_principal_ids: Some(known_principal_map_with_all_canisters.clone()),
        })
        .unwrap(),
    );

    let mut user_index_access_control_map = HashMap::new();
    user_index_access_control_map.insert(
        get_global_super_admin_principal_id(),
        vec![
            UserAccessRole::CanisterAdmin,
            UserAccessRole::CanisterController,
        ],
    );

    canister_installer(
        *known_principal_map_with_all_canisters
            .get(&KnownPrincipalType::CanisterIdUserIndex)
            .unwrap(),
        get_canister_wasm(KnownPrincipalType::CanisterIdUserIndex),
        candid::encode_one(UserIndexInitArgs {
            known_principal_ids: Some(known_principal_map_with_all_canisters.clone()),
            access_control_map: Some(user_index_access_control_map),
        })
        .unwrap(),
    );

    known_principal_map_with_all_canisters
}

pub fn get_canister_id_of_specific_type_from_principal_id_map(
    principal_id_map: &KnownPrincipalMap,
    canister_type: KnownPrincipalType,
) -> Principal {
    *principal_id_map
        .get(&canister_type)
        .expect("Canister type not found in principal id map")
}
