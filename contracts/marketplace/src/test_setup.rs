#[cfg(test)]
pub mod env {
    use cosmwasm_std::{Addr, Coin, Empty, Uint128};
    use cw20::Cw20Coin;
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    use crate::contract::{
        execute as MarketPlaceExecute, instantiate as MarketPlaceInstantiate,
        query as MarketPlaceQuery, reply as MarketplaceReply,
    };
    use crate::msg::InstantiateMsg;
    use cw721_base::entry::{
        execute as cw721Execute, instantiate as cw721Instantiate, query as cw721Query,
    };
    use cw721_base::msg::InstantiateMsg as cw721InstantiateMsg;

    // ****************************************
    // You MUST define the constants value here
    // ****************************************
    pub const OWNER: &str = "aura1000000000000000000000000000000000admin";
    pub const USER_1: &str = "aura1000000000000000000000000000000000user1";
    pub const USER_2: &str = "aura1000000000000000000000000000000000user2";

    pub const NATIVE_DENOM: &str = "uaura";
    pub const NATIVE_BALANCE: u128 = 1_000_000_000_000u128;

    pub const NATIVE_DENOM_2: &str = "uaura1";
    pub const NATIVE_BALANCE_2: u128 = 500_000_000_000u128;

    pub const TOKEN_INITIAL_BALANCE: u128 = 1_000_000_000_000u128;

    pub struct ContractInfo {
        pub contract_addr: String,
        pub contract_code_id: u64,
    }

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(OWNER),
                    vec![
                        Coin {
                            denom: NATIVE_DENOM.to_string(),
                            amount: Uint128::new(NATIVE_BALANCE),
                        },
                        Coin {
                            denom: NATIVE_DENOM_2.to_string(),
                            amount: Uint128::new(NATIVE_BALANCE_2),
                        },
                    ],
                )
                .unwrap();
        })
    }

    // *********************************************************
    // You MUST define the templates of all contracts here
    // Follow the example (1) below:
    //  pub fn contract_template() -> Box<dyn Contract<Empty>> {
    //      let contract = ContractWrapper::new(
    //          crate::contract::execute,
    //          crate::contract::instantiate,
    //          crate::contract::query,
    //      );
    //      Box::new(contract)
    //  }
    // *********************************************************
    fn cw721_contract_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(cw721Execute, cw721Instantiate, cw721Query);
        Box::new(contract)
    }

    fn nft_marketplace_contract_template() -> Box<dyn Contract<Empty>> {
        let contract =
            ContractWrapper::new(MarketPlaceExecute, MarketPlaceInstantiate, MarketPlaceQuery)
                .with_reply(MarketplaceReply);
        Box::new(contract)
    }

    fn cw20_contract_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            cw20_base::contract::execute,
            cw20_base::contract::instantiate,
            cw20_base::contract::query,
        );
        Box::new(contract)
    }

    // *********************************************************
    // You MUST store code and instantiate all contracts here
    // Follow the example (2) below:
    // @return App: the mock app
    // @return String: the address of the contract
    // @return u64: the code id of the contract
    //    pub fn instantiate_contracts() -> (App, String, u64) {
    //        // Create a new app instance
    //        let mut app = mock_app();
    //
    //        // store the code of all contracts to the app and get the code ids
    //        let contract_code_id = app.store_code(contract_template());
    //
    //        // create instantiate message for contract
    //        let contract_instantiate_msg = InstantiateMsg {
    //            name: "Contract_A".to_string(),
    //        };
    //
    //        // instantiate contract
    //        let contract_addr = app
    //            .instantiate_contract(
    //                contract_code_id,
    //                Addr::unchecked(OWNER),
    //                &contract_instantiate_msg,
    //                &[],
    //                "test instantiate contract",
    //                None,
    //            )
    //            .unwrap();
    //
    //        // return the app instance, the addresses and code IDs of all contracts
    //        (app, contract_addr, contract_code_id)
    //    }
    // *********************************************************
    pub fn instantiate_contracts() -> (App, Vec<ContractInfo>) {
        // Create a new app instance
        let mut app = mock_app();

        // Mint 1000000000 native token to USER_1
        app.sudo(cw_multi_test::SudoMsg::Bank(
            cw_multi_test::BankSudo::Mint {
                to_address: USER_1.to_string(),
                amount: vec![Coin {
                    amount: Uint128::from(1000000000u128),
                    denom: NATIVE_DENOM.to_string(),
                }],
            },
        ))
        .unwrap();

        // Mint 1000000000 native token to USER_2
        app.sudo(cw_multi_test::SudoMsg::Bank(
            cw_multi_test::BankSudo::Mint {
                to_address: USER_2.to_string(),
                amount: vec![Coin {
                    amount: Uint128::from(1000000000u128),
                    denom: NATIVE_DENOM.to_string(),
                }],
            },
        ))
        .unwrap();

        // Cw2981 contract
        // store the code of all contracts to the app and get the code ids
        let cw721_contract_code_id = app.store_code(cw721_contract_template());

        let mut contract_info_vec: Vec<ContractInfo> = Vec::new();

        // create instantiate message for contract
        let cw721_msg = cw721InstantiateMsg {
            name: "NFT_A".to_string(),
            symbol: "NFT".to_string(),
            minter: OWNER.to_string(),
        };

        // instantiate contract
        let cw721_contract_addr = app
            .instantiate_contract(
                cw721_contract_code_id,
                Addr::unchecked(OWNER),
                &cw721_msg,
                &[],
                "test instantiate cw2981 contract",
                None,
            )
            .unwrap();

        // add contract info to the vector
        contract_info_vec.push(ContractInfo {
            contract_addr: cw721_contract_addr.to_string(),
            contract_code_id: cw721_contract_code_id,
        });

        // NFT Marketplace contract
        // store the code of all contracts to the app and get the code ids
        let marketplace_contract_code_id = app.store_code(nft_marketplace_contract_template());

        // create instantiate message for contract
        let msg = InstantiateMsg {
            owner: Addr::unchecked(OWNER),
            collection_code_id: cw721_contract_code_id,
        };

        // instantiate contract
        let marketplace_contract_addr = app
            .instantiate_contract(
                marketplace_contract_code_id,
                Addr::unchecked(OWNER),
                &msg,
                &[],
                "test instantiate marketplace contract",
                None,
            )
            .unwrap();

        // add contract info to the vector
        contract_info_vec.push(ContractInfo {
            contract_addr: marketplace_contract_addr.to_string(),
            contract_code_id: marketplace_contract_code_id,
        });

        // CW20 contract
        // store the code of all contracts to the app and get the code ids
        let cw20_contract_code_id = app.store_code(cw20_contract_template());

        // create instantiate message for contract
        let cw20_msg = cw20_base::msg::InstantiateMsg {
            name: "CW20".to_string(),
            symbol: "CWA".to_string(),
            decimals: 6,
            initial_balances: vec![Cw20Coin {
                address: USER_2.to_string(),
                amount: Uint128::from(1_000_000_000_000u128),
            }],
            mint: None,
            marketing: None,
        };

        // instantiate contract
        let cw20_contract_addr = app
            .instantiate_contract(
                cw20_contract_code_id,
                Addr::unchecked(OWNER),
                &cw20_msg,
                &[],
                "test",
                None,
            )
            .unwrap();

        // add contract info to the vector
        contract_info_vec.push(ContractInfo {
            contract_addr: cw20_contract_addr.to_string(),
            contract_code_id: cw20_contract_code_id,
        });

        // return the app instance, the addresses and code IDs of all contracts
        (app, contract_info_vec)
    }
}
