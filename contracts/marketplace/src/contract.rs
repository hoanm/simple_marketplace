#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
};
use cw2::set_contract_version;
use cw_utils::parse_reply_instantiate_data;

use crate::error::ContractError;
use crate::execute::{
    execute_allow_payment_token, execute_buy, execute_cancel, execute_create_collection,
    execute_list_nft, execute_mint_nft,
};
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::query::{query_listing, query_listings_by_contract_address};
use crate::state::{ALLOWED_TOKENS, COLLECTIONS, COLLECTION_ID, CONFIG};
use crate::structs::Config;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:nft-marketplace";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let conf = Config {
        owner: msg.owner,
        collection_code_id: msg.collection_code_id,
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    CONFIG.save(deps.storage, &conf)?;

    COLLECTION_ID.save(deps.storage, &0u64)?;

    ALLOWED_TOKENS.save(deps.storage, &vec![])?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let _api = deps.api;
    match msg {
        ExecuteMsg::ListNft {
            asset,
            listing_config,
        } => execute_list_nft(deps, _env, info, asset, listing_config),
        ExecuteMsg::Buy { asset } => execute_buy(deps, _env, info, asset),
        ExecuteMsg::Cancel { asset } => execute_cancel(deps, _env, info, asset),
        ExecuteMsg::CreateCollection { name, symbol } => {
            execute_create_collection(deps, _env, info, name, symbol)
        }
        ExecuteMsg::MintNft {
            contract_address,
            token_id,
            token_uri,
        } => execute_mint_nft(deps, _env, info, contract_address, token_id, token_uri),
        ExecuteMsg::AllowPaymentToken { contract_address } => {
            execute_allow_payment_token(deps, _env, info, contract_address)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}

/// This just stores the result for future query
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    let reply = parse_reply_instantiate_data(msg.clone()).unwrap();

    let collection_contract = &reply.contract_address;

    // load minter from collections based on the msg.id
    let minter = COLLECTIONS.load(deps.storage, msg.id.to_string())?;

    // remove msg.id from collections
    COLLECTIONS.remove(deps.storage, msg.id.to_string());

    // save minter to collections
    COLLECTIONS.save(deps.storage, collection_contract.to_string(), &minter)?;
    Ok(Response::new().add_attributes(vec![
        ("action", "create_collection_reply"),
        ("collection_contract", collection_contract),
        ("minter", &minter),
    ]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let _api = deps.api;
    match msg {
        // get config
        QueryMsg::Config {} => to_json_binary(&CONFIG.load(deps.storage)?),
        QueryMsg::ListingsByContractAddress {
            contract_address,
            start_after,
            limit,
        } => to_json_binary(&query_listings_by_contract_address(
            deps,
            contract_address,
            start_after,
            limit,
        )?),
        QueryMsg::Listing {
            contract_address,
            token_id,
        } => to_json_binary(&query_listing(deps, contract_address, token_id)?),
    }
}
