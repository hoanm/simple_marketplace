use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, BlockInfo, Coin};
use cw721::Expiration;
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, MultiIndex};

pub type TokenId = String;

#[cw_serde]
pub struct ListingConfig {
    pub price: Coin,
    pub start_time: Option<Expiration>, // we use expiration for convinience
    pub end_time: Option<Expiration>,   // it's required that start_time < end_time
}

#[cw_serde]
pub struct Listing {
    pub contract_address: Addr, // contract contains the NFT
    pub token_id: String,       // id of the NFT
    pub listing_config: ListingConfig,
    pub seller: Addr,
    pub buyer: Option<Addr>, // buyer, will be initialized to None
}

impl Listing {
    // expired is when a listing has passed the end_time
    pub fn is_expired(&self, block_info: &BlockInfo) -> bool {
        match self.listing_config.end_time {
            Some(time) => time.is_expired(block_info),
            None => false,
        }
    }
}

// ListingKey is unique for all listings
pub type ListingKey = (Addr, TokenId);

pub fn listing_key(contract_address: &Addr, token_id: &TokenId) -> ListingKey {
    (contract_address.clone(), token_id.clone())
}

// listings can be indexed by contract_address
// contract_address can point to multiple listings
pub struct ListingIndexes<'a> {
    pub contract_address: MultiIndex<'a, Addr, Listing, ListingKey>,
}

impl<'a> IndexList<Listing> for ListingIndexes<'a> {
    // this method returns a list of all indexes
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Listing>> + '_> {
        let v: Vec<&dyn Index<Listing>> = vec![&self.contract_address];
        Box::new(v.into_iter())
    }
}

// helper function create a IndexedMap for listings
pub fn listings<'a>() -> IndexedMap<'a, ListingKey, Listing, ListingIndexes<'a>> {
    let indexes = ListingIndexes {
        contract_address: MultiIndex::new(
            |_pk: &[u8], l: &Listing| (l.contract_address.clone()),
            "listings",
            "listings__contract_address",
        ),
    };
    IndexedMap::new("listings", indexes)
}

#[cw_serde]
pub struct Config {
    pub owner: Addr,
}

// contract class is a wrapper for all storage items
pub struct MarketplaceContract<'a> {
    pub config: Item<'a, Config>,
    pub listings: IndexedMap<'a, ListingKey, Listing, ListingIndexes<'a>>,
}

// impl default for MarketplaceContract
impl Default for MarketplaceContract<'static> {
    fn default() -> Self {
        MarketplaceContract {
            config: Item::<Config>::new("config"),
            listings: listings(),
        }
    }
}

// public the default MarketplaceContract
pub fn contract() -> MarketplaceContract<'static> {
    MarketplaceContract::default()
}
