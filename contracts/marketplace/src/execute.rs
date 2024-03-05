use crate::{
    error::ContractError,
    state::{listing_key, ALLOWED_TOKENS, COLLECTIONS, COLLECTION_ID, CONFIG, LISTINGS},
    structs::{
        order_id, Asset, AuctionConfig, ConsiderationItem, Cw20Asset, ItemType, NativeAsset,
        NftAsset, OfferItem, Order, OrderType, PaymentAsset,
    },
};
use cosmwasm_std::{
    to_json_binary, Addr, BankMsg, Coin, CosmosMsg, DepsMut, Empty, Env, MessageInfo, QueryRequest,
    ReplyOn, Response, StdResult, SubMsg, Uint128, WasmMsg, WasmQuery,
};
use cw20::Cw20ExecuteMsg;
use cw721::{Cw721ExecuteMsg, Cw721QueryMsg, Expiration as Cw721Expiration};
use cw721_base::{Extension, InstantiateMsg as Cw721InstantiateMsg};

pub fn execute_list_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    asset: NftAsset,
    auction_config: AuctionConfig,
) -> Result<Response, ContractError> {
    // auction time must be valid first
    if !auction_config.is_valid() {
        return Err(ContractError::CustomError {
            val: "Invalid listing config".to_string(),
        });
    }

    let contract_address = asset.contract_address.clone();
    // token_id is required
    if asset.token_id.is_none() {
        return Err(ContractError::CustomError {
            val: "Token ID is required".to_string(),
        });
    }
    let token_id = asset.token_id.unwrap();

    // check if user is the owner of the token
    let owner_response: StdResult<cw721::OwnerOfResponse> =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: contract_address.to_string(),
            msg: to_json_binary(&Cw721QueryMsg::OwnerOf {
                token_id: token_id.clone(),
                include_expired: Some(false),
            })?,
        }));
    match owner_response {
        Ok(owner) => {
            if owner.owner != info.sender {
                return Err(ContractError::Unauthorized {});
            }
        }
        Err(_) => {
            return Err(ContractError::Unauthorized {});
        }
    }

    // check that user approves this contract to manage this token
    // for now, we require never expired approval
    let approval_response: StdResult<cw721::ApprovalResponse> =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: contract_address.to_string(),
            msg: to_json_binary(&Cw721QueryMsg::Approval {
                token_id: token_id.clone(),
                spender: env.contract.address.to_string(),
                include_expired: Some(true),
            })?,
        }));
    match approval_response {
        Ok(approval) => match approval.approval.expires {
            Cw721Expiration::Never {} => {}
            _ => return Err(ContractError::Unauthorized {}),
        },
        Err(_) => {
            return Err(ContractError::CustomError {
                val: "Require never expired approval".to_string(),
            });
        }
    }

    // the auction_config must be FixedPrice
    match auction_config.clone() {
        AuctionConfig::FixedPrice {
            price,
            start_time,
            end_time,
        } => {
            // add new listing to orders
            let order_id = order_id(&info.sender, &contract_address, &token_id);

            let offer_item = OfferItem {
                item_type: ItemType::CW721,
                item: Asset::Nft(NftAsset {
                    contract_address: contract_address.clone(),
                    token_id: Some(token_id.clone()),
                }),
                start_amount: 1,
                end_amount: 1,
            };

            let consideration_item = match price {
                PaymentAsset::Native { denom, amount } => ConsiderationItem {
                    item_type: ItemType::NATIVE,
                    item: Asset::Native(NativeAsset { denom, amount }),
                    start_amount: amount,
                    end_amount: amount,
                    recipient: info.sender.clone(),
                },
                PaymentAsset::Cw20 {
                    contract_address,
                    amount,
                } => {
                    // check if contract_address is in ALLOWED_TOKENS
                    let allowed_tokens: Vec<Addr> = ALLOWED_TOKENS.load(deps.storage)?;
                    if !allowed_tokens.contains(&contract_address) {
                        return Err(ContractError::CustomError {
                            val: "Payment token not allowed".to_string(),
                        });
                    }
                    ConsiderationItem {
                        item_type: ItemType::CW20,
                        item: Asset::Cw20(Cw20Asset {
                            contract_address,
                            amount,
                        }),
                        start_amount: amount,
                        end_amount: amount,
                        recipient: info.sender.clone(),
                    }
                }
            };

            let new_listing = Order {
                order_type: OrderType::LISTING,
                order_id,
                owner: info.sender.clone(),
                offer: vec![offer_item],
                consideration: vec![consideration_item],
                start_time,
                end_time,
            };

            let listing_key = listing_key(&contract_address, &token_id);
            // we will override the order if it already exists, so that we can update the auction config
            LISTINGS.update(
                deps.storage,
                listing_key,
                |_old| -> Result<Order, ContractError> { Ok(new_listing) },
            )?;
        }
        _ => {
            return Err(ContractError::CustomError {
                val: "Auction Config Error".to_string(),
            });
        }
    }

    let auction_config_str = serde_json::to_string(&auction_config);

    match auction_config_str {
        Ok(auction_config_str) => Ok(Response::new()
            .add_attribute("method", "list_nft")
            .add_attribute("contract_address", contract_address)
            .add_attribute("token_id", token_id)
            .add_attribute("auction_config", auction_config_str)
            .add_attribute("seller", info.sender.to_string())),
        Err(_) => Err(ContractError::CustomError {
            val: ("Auction Config Error".to_string()),
        }),
    }
}

pub fn execute_buy(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    asset: NftAsset,
) -> Result<Response, ContractError> {
    let contract_address = asset.contract_address.clone();
    // token_id is required
    if asset.token_id.is_none() {
        return Err(ContractError::CustomError {
            val: "Token ID is required".to_string(),
        });
    }
    let token_id = asset.token_id.unwrap();

    // get the listing
    let listing_key = listing_key(&contract_address, &token_id);
    let listing = LISTINGS.load(deps.storage, listing_key.clone())?;

    // check if owner of listing is the same as seller
    if info.sender == listing.owner {
        return Err(ContractError::CustomError {
            val: ("Owner cannot buy".to_string()),
        });
    }

    // remove the listing
    LISTINGS.remove(deps.storage, listing_key)?;

    // check if current block is after start_time
    if listing.start_time.is_some() && !listing.start_time.unwrap().is_expired(&env.block) {
        return Err(ContractError::CustomError {
            val: ("Auction not started".to_string()),
        });
    }

    if listing.end_time.is_some() && listing.end_time.unwrap().is_expired(&env.block) {
        return Err(ContractError::CustomError {
            val: format!(
                "Auction ended: {} {}",
                listing.end_time.unwrap(),
                env.block.time
            ),
        });
    }

    // message to transfer nft to buyer
    let mut res = Response::new().add_message(WasmMsg::Execute {
        contract_addr: contract_address.to_string(),
        msg: to_json_binary(&Cw721ExecuteMsg::TransferNft {
            recipient: info.sender.to_string(),
            token_id: token_id.clone(),
        })?,
        funds: vec![],
    });

    // transfer payment assets to the recipient of listing's consideration
    let payment_messages = payment_processing(
        &deps,
        &info,
        &PaymentAsset::from(listing.consideration[0].item.clone()),
        &info.sender,
        &listing.consideration[0].recipient,
    )?;

    for payment_message in payment_messages {
        res = res.add_message(payment_message);
    }

    Ok(res
        .add_attribute("method", "buy")
        .add_attribute("contract_address", contract_address.to_string())
        .add_attribute("token_id", token_id.to_string())
        .add_attribute("buyer", info.sender))
}

pub fn execute_cancel(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    asset: NftAsset,
) -> Result<Response, ContractError> {
    let contract_address = asset.contract_address.clone();
    // token_id is required
    if asset.token_id.is_none() {
        return Err(ContractError::CustomError {
            val: "Token ID is required".to_string(),
        });
    }
    let token_id = asset.token_id.unwrap();

    // find listing
    let listing_key = listing_key(&contract_address, &token_id);
    let listing = LISTINGS.load(deps.storage, listing_key.clone())?;

    // if a listing is not expired, only seller can cancel
    if (!listing.is_expired(&env.block)) && (listing.owner != info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // we will remove the cancelled listing
    LISTINGS.remove(deps.storage, listing_key)?;

    Ok(Response::new()
        .add_attribute("method", "cancel")
        .add_attribute("contract_address", contract_address)
        .add_attribute("token_id", token_id)
        .add_attribute("cancelled_at", env.block.time.to_string()))
}

pub fn execute_create_collection(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    name: String,
    symbol: String,
) -> Result<Response, ContractError> {
    // load config
    let config = CONFIG.load(deps.storage)?;
    // load collection_id
    let mut collection_id = COLLECTION_ID.load(deps.storage)?;
    // increment collection_id
    collection_id += 1;
    // save collection_id

    // save a temporary collection_id to storage
    COLLECTIONS.save(
        deps.storage,
        collection_id.to_string(),
        &info.sender.to_string(),
    )?;

    COLLECTION_ID.save(deps.storage, &collection_id)?;
    Ok(Response::new()
        .add_submessage(SubMsg {
            id: collection_id,
            gas_limit: None,
            msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
                code_id: config.collection_code_id,
                funds: vec![],
                admin: Some(env.contract.address.to_string()),
                label: "create collection".to_string(),
                msg: to_json_binary(&Cw721InstantiateMsg {
                    name: name.clone(),
                    symbol: symbol.clone(),
                    minter: env.contract.address.to_string(),
                })?,
            }),
            reply_on: ReplyOn::Success,
        })
        .add_attribute("method", "create_collection")
        .add_attribute("name", name)
        .add_attribute("symbol", symbol)
        .add_attribute("minter", info.sender.to_string()))
}

pub fn execute_mint_nft(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    contract_address: String,
    token_id: String,
    token_uri: String,
) -> Result<Response, ContractError> {
    // check if contract address and info.sender are valid
    let collection_minter = COLLECTIONS.load(deps.storage, contract_address.to_string())?;
    if collection_minter != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // mint nft
    let transfer_nft_msg = WasmMsg::Execute {
        contract_addr: contract_address,
        msg: to_json_binary(&cw721_base::ExecuteMsg::<Extension, Empty>::Mint {
            token_id: token_id.clone(),
            owner: info.sender.to_string(),
            token_uri: Some(token_uri.clone()),
            extension: None,
        })?,
        funds: vec![],
    };

    Ok(Response::new()
        .add_message(transfer_nft_msg)
        .add_attribute("action", "mint_nft")
        .add_attribute("minter", info.sender)
        .add_attribute("token_id", token_id)
        .add_attribute("token_uri", token_uri))
}

// function to process payment transfer
fn payment_processing(
    deps: &DepsMut,
    info: &MessageInfo,
    asset: &PaymentAsset,
    sender: &Addr,
    recipient: &Addr,
) -> Result<Vec<CosmosMsg>, ContractError> {
    // create empty vector of CosmosMsg
    let mut res_messages: Vec<CosmosMsg> = vec![];

    // Extract information from token
    let (is_native, token_info, amount) = match asset {
        PaymentAsset::Cw20 {
            contract_address,
            amount,
        } => (
            false,
            (*contract_address).to_string().clone(),
            Uint128::from(*amount),
        ),
        PaymentAsset::Native { denom, amount } => (true, (*denom).clone(), Uint128::from(*amount)),
    };

    match &is_native {
        false => {
            // execute cw20 transfer msg from info.sender to recipient
            let transfer_response = WasmMsg::Execute {
                contract_addr: deps.api.addr_validate(&token_info).unwrap().to_string(),
                msg: to_json_binary(&Cw20ExecuteMsg::TransferFrom {
                    owner: sender.to_string(),
                    recipient: recipient.to_string(),
                    amount,
                })
                .unwrap(),
                funds: vec![],
            };
            res_messages.push(transfer_response.into());
        }
        true => {
            let price = Coin {
                denom: token_info.to_string(),
                amount,
            };
            // check if enough funds
            if info.funds.is_empty() || info.funds[0] != price {
                return Err(ContractError::InsufficientFunds {});
            }
            // transfer all funds to recipient
            let transfer_response = BankMsg::Send {
                to_address: recipient.to_string(),
                amount: vec![price],
            };
            res_messages.push(transfer_response.into());
        }
    }

    Ok(res_messages)
}

pub fn execute_allow_payment_token(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    contract_address: Addr,
) -> Result<Response, ContractError> {
    // check if sender is the owner
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    // check if contract_address is in ALLOWED_TOKENS
    let mut allowed_tokens: Vec<Addr> = ALLOWED_TOKENS.load(deps.storage)?;
    if !allowed_tokens.contains(&contract_address) {
        allowed_tokens.push(contract_address.clone());
        ALLOWED_TOKENS.save(deps.storage, &allowed_tokens)?;
    }

    Ok(Response::new()
        .add_attribute("action", "allow_payment_token")
        .add_attribute("contract_address", contract_address))
}
