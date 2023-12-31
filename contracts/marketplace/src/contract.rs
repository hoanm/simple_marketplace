#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
};
use cw2::set_contract_version;
use cw_utils::parse_reply_instantiate_data;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{contract, Config, COLLECTION_ID};

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
    // the default value of vaura_address is equal to "aura0" and MUST BE SET before offer nft
    let conf = Config {
        owner: msg.owner,
        collection_code_id: msg.collection_code_id,
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    contract().config.save(deps.storage, &conf)?;

    COLLECTION_ID.save(deps.storage, &0u64)?;

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
            contract_address,
            token_id,
            listing_config,
        } => contract().execute_list_nft(
            deps,
            _env,
            info,
            contract_address,
            token_id,
            listing_config,
        ),
        ExecuteMsg::Buy {
            contract_address,
            token_id,
        } => contract().execute_buy(deps, _env, info, contract_address, token_id),
        ExecuteMsg::Cancel {
            contract_address,
            token_id,
        } => contract().execute_cancel(deps, _env, info, contract_address, token_id),
        ExecuteMsg::CreateCollection { name, symbol } => {
            contract().execute_create_collection(deps, _env, info, name, symbol)
        }
        ExecuteMsg::MintNft {
            contract_address,
            token_id,
            token_uri,
        } => contract().execute_mint_nft(deps, _env, info, contract_address, token_id, token_uri),
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

    // load minter from contract().collections based on the msg.id
    let minter = contract()
        .collections
        .load(deps.storage, msg.id.to_string())?;

    // remove msg.id from contract().collections
    contract()
        .collections
        .remove(deps.storage, msg.id.to_string());

    // save minter to contract().collections
    contract()
        .collections
        .save(deps.storage, collection_contract.to_string(), &minter)?;
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
        QueryMsg::Config {} => to_binary(&contract().config.load(deps.storage)?),
        QueryMsg::ListingsByContractAddress {
            contract_address,
            start_after,
            limit,
        } => to_binary(&contract().query_listings_by_contract_address(
            deps,
            contract_address,
            start_after,
            limit,
        )?),
        QueryMsg::Listing {
            contract_address,
            token_id,
        } => to_binary(&contract().query_listing(deps, contract_address, token_id)?),
    }
}
