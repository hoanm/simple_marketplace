use crate::msg::ExecuteMsg;

use crate::test_setup::env::{instantiate_contracts, USER_1};

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
    use cw_multi_test::Executor;

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
            contract_address: "contract2".to_string(),
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
        println!("res: {:?}", res);
        assert!(res.is_ok());
    }
}
