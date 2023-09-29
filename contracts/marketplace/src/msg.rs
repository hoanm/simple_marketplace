use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

use crate::state::{Listing, ListingConfig};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Addr,
    pub collection_code_id: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    // List a NFT for sale
    ListNft {
        contract_address: String,
        token_id: String,
        listing_config: ListingConfig,
    },
    // Buy a listed NFT
    Buy {
        contract_address: String,
        token_id: String,
    },
    // Cancel a listed NFT
    Cancel {
        contract_address: String,
        token_id: String,
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
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // list config of contract
    #[returns(crate::state::Config)]
    Config {},
    // get listing by contract_address
    #[returns(ListingsResponse)]
    ListingsByContractAddress {
        contract_address: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    // get listing by contract_address and token_id
    #[returns(Listing)]
    Listing {
        contract_address: String,
        token_id: String,
    },
}

#[cw_serde]
pub struct ListingsResponse {
    pub listings: Vec<Listing>,
}
