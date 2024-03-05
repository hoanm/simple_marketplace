use crate::msg::ExecuteMsg;

use crate::test_setup::env::{instantiate_contracts, USER_1, USER_2};
use crate::{
    structs::{AuctionConfig, NftAsset, PaymentAsset},
    test_setup::env::NATIVE_DENOM,
};
use cosmwasm_std::BalanceResponse as BankBalanceResponse;
use cosmwasm_std::{from_json, to_json_binary, BankQuery, QueryRequest};
use cw_multi_test::Executor;

use cosmwasm_std::Addr;

// const MOCK_CW721_ADDR: &str = "cw721_addr";
// const MOCK_OFFER_NFT_TOKEN_ID_1: &str = "1";
// const MOCK_OFFER_NFT_TOKEN_ID_INVALID: &str = "invalid_id";

// const MOCK_OFFER_NFT_OWNER: &str = "owner";
// const MOCK_OFFER_NFT_CREATOR: &str = "creator";
// const MOCK_OFFER_NFT_OFFERER_1: &str = "offerer 1";
// const MOCK_OFFER_NFT_OFFERER_INSUFFICIENT_BALANCE: &str = "offerer 2";
// const MOCK_OFFER_NFT_OFFERER_INSUFFICIENT_ALLOWANCE: &str = "offerer 3";

mod create_collection {
    use super::*;

    #[test]
    fn user_can_create_own_collection() {
        // get integration test app and contracts
        let (mut app, contracts) = instantiate_contracts();
        let _cw721_address = contracts[0].contract_addr.clone();
        let marketplace_address = contracts[1].contract_addr.clone();

        // prepare create collection message
        let create_collection_msg = ExecuteMsg::CreateCollection {
            name: "NFT_A".to_string(),
            symbol: "NFT".to_string(),
        };

        // USER_1 creates collection
        let res = app.execute_contract(
            Addr::unchecked(USER_1.to_string()),
            Addr::unchecked(marketplace_address),
            &create_collection_msg,
            &[],
        );
        assert!(res.is_ok());
    }

    #[test]
    fn user_can_mint_token_with_created_collection() {
        // get integration test app and contracts
        let (mut app, contracts) = instantiate_contracts();
        let cw721_address = contracts[0].contract_addr.clone();
        let marketplace_address = contracts[1].contract_addr.clone();

        println!("cw721_address: {:?}", cw721_address);
        println!("marketplace_address: {:?}", marketplace_address);

        // prepare create collection message
        let create_collection_msg = ExecuteMsg::CreateCollection {
            name: "NFT_A".to_string(),
            symbol: "NFT".to_string(),
        };

        // USER_1 creates collection
        let res = app.execute_contract(
            Addr::unchecked(USER_1.to_string()),
            Addr::unchecked(marketplace_address.clone()),
            &create_collection_msg,
            &[],
        );
        // println!("res: {:?}", res);
        assert!(res.is_ok());

        // prepare mint token message
        let mint_token_msg = ExecuteMsg::MintNft {
            contract_address: "contract3".to_string(),
            token_id: "1".to_string(),
            token_uri: "https://www.google.com".to_string(),
        };

        // USER_1 mints token
        let res = app.execute_contract(
            Addr::unchecked(USER_1.to_string()),
            Addr::unchecked(marketplace_address),
            &mint_token_msg,
            &[],
        );
        assert!(res.is_ok());
    }
}

mod listing_nft {
    use cosmwasm_std::{Coin, Uint128};
    use cw20::BalanceResponse;

    use crate::test_setup::env::OWNER;

    use super::*;

    #[test]
    fn user_can_list_nft_for_sale() {
        // get integration test app and contracts
        let (mut app, contracts) = instantiate_contracts();
        // let cw721_address = contracts[0].contract_addr.clone();
        let marketplace_address = contracts[1].contract_addr.clone();

        // prepare create collection message
        let create_collection_msg = ExecuteMsg::CreateCollection {
            name: "NFT_A".to_string(),
            symbol: "NFT".to_string(),
        };

        // USER_1 creates collection
        let res = app.execute_contract(
            Addr::unchecked(USER_1.to_string()),
            Addr::unchecked(marketplace_address.clone()),
            &create_collection_msg,
            &[],
        );
        assert!(res.is_ok());

        // prepare mint token message
        let mint_token_msg = ExecuteMsg::MintNft {
            contract_address: "contract3".to_string(),
            token_id: "1".to_string(),
            token_uri: "https://www.google.com".to_string(),
        };

        // USER_1 mints token
        let res = app.execute_contract(
            Addr::unchecked(USER_1.to_string()),
            Addr::unchecked(marketplace_address.clone()),
            &mint_token_msg,
            &[],
        );
        assert!(res.is_ok());

        // prepare list nft message
        let list_nft_msg = ExecuteMsg::ListNft {
            asset: NftAsset {
                contract_address: Addr::unchecked("contract3".to_string()),
                token_id: Some("1".to_string()),
            },
            listing_config: AuctionConfig::FixedPrice {
                price: PaymentAsset::Native {
                    denom: NATIVE_DENOM.to_string(),
                    amount: 100u128.into(),
                },
                start_time: None,
                end_time: None,
            },
        };

        // USER_1 approves marketplace to spend nft
        let res = app.execute_contract(
            Addr::unchecked(USER_1.to_string()),
            Addr::unchecked("contract3".to_string()),
            &cw721::Cw721ExecuteMsg::Approve {
                spender: marketplace_address.clone(),
                token_id: "1".to_string(),
                expires: None,
            },
            &[],
        );
        assert!(res.is_ok());

        // USER_1 lists nft for sale
        let res = app.execute_contract(
            Addr::unchecked(USER_1.to_string()),
            Addr::unchecked(marketplace_address),
            &list_nft_msg,
            &[],
        );
        assert!(res.is_ok());
    }

    #[test]
    fn user_can_listing_and_buying_nft() {
        // get integration test app and contracts
        let (mut app, contracts) = instantiate_contracts();
        // let cw721_address = contracts[0].contract_addr.clone();
        let marketplace_address = contracts[1].contract_addr.clone();

        // prepare create collection message
        let create_collection_msg = ExecuteMsg::CreateCollection {
            name: "NFT_A".to_string(),
            symbol: "NFT".to_string(),
        };

        // USER_1 creates collection
        let res = app.execute_contract(
            Addr::unchecked(USER_1.to_string()),
            Addr::unchecked(marketplace_address.clone()),
            &create_collection_msg,
            &[],
        );
        assert!(res.is_ok());

        // prepare mint token message
        let mint_token_msg = ExecuteMsg::MintNft {
            contract_address: "contract3".to_string(),
            token_id: "1".to_string(),
            token_uri: "https://www.google.com".to_string(),
        };

        // USER_1 mints token
        let res = app.execute_contract(
            Addr::unchecked(USER_1.to_string()),
            Addr::unchecked(marketplace_address.clone()),
            &mint_token_msg,
            &[],
        );
        assert!(res.is_ok());

        // prepare list nft message
        let list_nft_msg = ExecuteMsg::ListNft {
            asset: NftAsset {
                contract_address: Addr::unchecked("contract3".to_string()),
                token_id: Some("1".to_string()),
            },
            listing_config: AuctionConfig::FixedPrice {
                price: PaymentAsset::Native {
                    denom: NATIVE_DENOM.to_string(),
                    amount: 100u128.into(),
                },
                start_time: None,
                end_time: None,
            },
        };

        // USER_1 approves marketplace to spend nft
        let res = app.execute_contract(
            Addr::unchecked(USER_1.to_string()),
            Addr::unchecked("contract3".to_string()),
            &cw721::Cw721ExecuteMsg::Approve {
                spender: marketplace_address.clone(),
                token_id: "1".to_string(),
                expires: None,
            },
            &[],
        );
        assert!(res.is_ok());

        // USER_1 lists nft for sale
        let res = app.execute_contract(
            Addr::unchecked(USER_1.to_string()),
            Addr::unchecked(marketplace_address.clone()),
            &list_nft_msg,
            &[],
        );
        assert!(res.is_ok());

        // prepare buy nft message
        let buy_nft_msg = ExecuteMsg::Buy {
            asset: NftAsset {
                contract_address: Addr::unchecked("contract3".to_string()),
                token_id: Some("1".to_string()),
            },
        };

        // Query balance of USER_1
        let req: QueryRequest<BankQuery> = QueryRequest::Bank(BankQuery::Balance {
            address: USER_1.to_string(),
            denom: NATIVE_DENOM.to_string(),
        });
        let res = app
            .wrap()
            .raw_query(&to_json_binary(&req).unwrap())
            .unwrap()
            .unwrap();
        let balance_before: BankBalanceResponse = from_json(&res).unwrap();

        // USER_2 buys nft
        let res = app.execute_contract(
            Addr::unchecked(USER_2.to_string()),
            Addr::unchecked(marketplace_address),
            &buy_nft_msg,
            &[Coin {
                denom: NATIVE_DENOM.to_string(),
                amount: 100u128.into(),
            }],
        );
        assert!(res.is_ok());

        // Query balance of USER_1
        let req: QueryRequest<BankQuery> = QueryRequest::Bank(BankQuery::Balance {
            address: USER_1.to_string(),
            denom: NATIVE_DENOM.to_string(),
        });
        let res = app
            .wrap()
            .raw_query(&to_json_binary(&req).unwrap())
            .unwrap()
            .unwrap();
        let balance_after: BankBalanceResponse = from_json(&res).unwrap();
        assert_eq!(
            balance_after
                .amount
                .amount
                .checked_sub(Uint128::from(100u128))
                .unwrap(),
            balance_before.amount.amount
        );
    }

    #[test]
    fn user_can_listing_and_buying_using_cw20_token() {
        // get integration test app and contracts
        let (mut app, contracts) = instantiate_contracts();
        let cw20_address = contracts[2].contract_addr.clone();
        let marketplace_address = contracts[1].contract_addr.clone();

        // prepare create collection message
        let create_collection_msg = ExecuteMsg::CreateCollection {
            name: "NFT_A".to_string(),
            symbol: "NFT".to_string(),
        };

        // USER_1 creates collection
        let res = app.execute_contract(
            Addr::unchecked(USER_1.to_string()),
            Addr::unchecked(marketplace_address.clone()),
            &create_collection_msg,
            &[],
        );
        assert!(res.is_ok());

        // prepare mint token message
        let mint_token_msg = ExecuteMsg::MintNft {
            contract_address: "contract3".to_string(),
            token_id: "1".to_string(),
            token_uri: "https://www.google.com".to_string(),
        };

        // USER_1 mints token
        let res = app.execute_contract(
            Addr::unchecked(USER_1.to_string()),
            Addr::unchecked(marketplace_address.clone()),
            &mint_token_msg,
            &[],
        );
        assert!(res.is_ok());

        // prepare list nft message
        let list_nft_msg = ExecuteMsg::ListNft {
            asset: NftAsset {
                contract_address: Addr::unchecked("contract3".to_string()),
                token_id: Some("1".to_string()),
            },
            listing_config: AuctionConfig::FixedPrice {
                price: PaymentAsset::Cw20 {
                    contract_address: Addr::unchecked(cw20_address.clone()),
                    amount: 100u128,
                },
                start_time: None,
                end_time: None,
            },
        };

        // USER_1 approves marketplace to spend nft
        let res = app.execute_contract(
            Addr::unchecked(USER_1.to_string()),
            Addr::unchecked("contract3".to_string()),
            &cw721::Cw721ExecuteMsg::Approve {
                spender: marketplace_address.clone(),
                token_id: "1".to_string(),
                expires: None,
            },
            &[],
        );
        assert!(res.is_ok());

        // USER_1 lists nft for sale
        let res = app.execute_contract(
            Addr::unchecked(USER_1.to_string()),
            Addr::unchecked(marketplace_address.clone()),
            &list_nft_msg,
            &[],
        );
        assert!(res.is_err());

        // OWNER allows cw20 token to be used for payment
        let res = app.execute_contract(
            Addr::unchecked(OWNER.to_string()),
            Addr::unchecked(marketplace_address.clone()),
            &ExecuteMsg::AllowPaymentToken {
                contract_address: Addr::unchecked(cw20_address.clone()),
            },
            &[],
        );
        assert!(res.is_ok());

        // USER_1 lists nft for sale
        let res = app.execute_contract(
            Addr::unchecked(USER_1.to_string()),
            Addr::unchecked(marketplace_address.clone()),
            &list_nft_msg,
            &[],
        );
        assert!(res.is_ok());

        // prepare buy nft message
        let buy_nft_msg = ExecuteMsg::Buy {
            asset: NftAsset {
                contract_address: Addr::unchecked("contract3".to_string()),
                token_id: Some("1".to_string()),
            },
        };

        // Query balance cw20 token of USER_2
        let balance_before: BalanceResponse = app
            .wrap()
            .query_wasm_smart(
                cw20_address.clone(),
                &cw20::Cw20QueryMsg::Balance {
                    address: USER_2.to_string(),
                },
            )
            .unwrap();

        // USER_2 allows marketplace to spend cw20 token
        let res = app.execute_contract(
            Addr::unchecked(USER_2.to_string()),
            Addr::unchecked(cw20_address.clone()),
            &cw20::Cw20ExecuteMsg::IncreaseAllowance {
                spender: marketplace_address.clone(),
                amount: 100u128.into(),
                expires: None,
            },
            &[],
        );
        assert!(res.is_ok());

        // USER_2 buys nft
        let res = app.execute_contract(
            Addr::unchecked(USER_2.to_string()),
            Addr::unchecked(marketplace_address),
            &buy_nft_msg,
            &[],
        );
        assert!(res.is_ok());

        // Query balance cw20 token of USER_2
        let balance_after: BalanceResponse = app
            .wrap()
            .query_wasm_smart(
                cw20_address.clone(),
                &cw20::Cw20QueryMsg::Balance {
                    address: USER_2.to_string(),
                },
            )
            .unwrap();
        assert_eq!(
            balance_after.balance,
            balance_before
                .balance
                .checked_sub(Uint128::from(100u128))
                .unwrap()
        );

        // Query balance cw20 token of USER_1
        let balance: BalanceResponse = app
            .wrap()
            .query_wasm_smart(
                cw20_address.clone(),
                &cw20::Cw20QueryMsg::Balance {
                    address: USER_1.to_string(),
                },
            )
            .unwrap();

        assert_eq!(balance.balance, Uint128::from(100u128));
    }
}
