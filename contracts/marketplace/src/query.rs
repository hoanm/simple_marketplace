use cosmwasm_std::{Deps, Order, StdResult};
use cw_storage_plus::Bound;

use crate::{
    msg::ListingsResponse,
    state::{listing_key, Listing, ListingKey, MarketplaceContract},
};

impl MarketplaceContract<'static> {
    pub fn query_listing(
        self,
        deps: Deps,
        contract_address: String,
        token_id: String,
    ) -> StdResult<Listing> {
        let listing_key = listing_key(contract_address, &token_id);
        self.listings.load(deps.storage, listing_key)
    }

    pub fn query_listings_by_contract_address(
        self,
        deps: Deps,
        contract_address: String,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> StdResult<ListingsResponse> {
        let limit = limit.unwrap_or(30).min(30) as usize;
        let start: Option<Bound<ListingKey>> = start_after
            .map(|token_id| Bound::exclusive(listing_key(contract_address.clone(), &token_id)));
        let listings = self
            .listings
            .idx
            .contract_address
            .prefix(contract_address)
            .range(deps.storage, start, None, Order::Ascending)
            .map(|item| item.map(|(_, listing)| listing))
            .take(limit)
            .collect::<StdResult<Vec<_>>>()?;
        Ok(ListingsResponse { listings })
    }
}
