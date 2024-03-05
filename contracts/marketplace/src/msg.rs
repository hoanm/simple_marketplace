use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

use crate::structs::{AuctionConfig, Config, ListingsResponse, NftAsset, Order};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Addr,
    pub collection_code_id: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    // List a NFT for sale
    ListNft {
        asset: NftAsset,
        listing_config: AuctionConfig,
    },
    // Buy a listed NFT
    Buy {
        asset: NftAsset,
    },
    // Cancel a listed NFT
    Cancel {
        asset: NftAsset,
    },
    // User creates a new collection
    CreateCollection {
        name: String,
        symbol: String,
    },
    // User mints a new NFT
    MintNft {
        contract_address: String,
        token_id: String,
        token_uri: String,
    },
    // Admin allows payment token to be used for payment
    AllowPaymentToken {
        contract_address: Addr,
    },
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // list config of contract
    #[returns(Config)]
    Config {},
    // get listing by contract_address
    #[returns(ListingsResponse)]
    ListingsByContractAddress {
        contract_address: Addr,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    // get listing by contract_address and token_id
    #[returns(Order)]
    Listing {
        contract_address: Addr,
        token_id: String,
    },
}
