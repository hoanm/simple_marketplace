use cosmwasm_std::{Addr, Deps, Order, StdResult};
use cw_storage_plus::Bound;

use crate::{
    state::{listing_key, ListingKey, LISTINGS},
    structs::{ListingsResponse, Order as Listing},
};

pub fn query_listing(deps: Deps, contract_address: Addr, token_id: String) -> StdResult<Listing> {
    let listing_key = listing_key(&contract_address, &token_id);
    LISTINGS.load(deps.storage, listing_key)
}

pub fn query_listings_by_contract_address(
    deps: Deps,
    contract_address: Addr,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<ListingsResponse> {
    let limit = limit.unwrap_or(30).min(30) as usize;
    let start: Option<Bound<ListingKey>> =
        start_after.map(|token_id| Bound::exclusive(listing_key(&contract_address, &token_id)));
    let listings = LISTINGS
        .idx
        .contract_address
        .prefix(contract_address)
        .range(deps.storage, start, None, Order::Ascending)
        .map(|item| item.map(|(_, listing)| listing))
        .take(limit)
        .collect::<StdResult<Vec<_>>>()?;
    Ok(ListingsResponse { listings })
}
