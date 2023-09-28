use crate::{
    state::{listing_key, Listing, ListingConfig, MarketplaceContract},
    ContractError,
};
use cosmwasm_std::{
    to_binary, Addr, DepsMut, Env, MessageInfo, QueryRequest, Response, StdResult, WasmMsg,
    WasmQuery,
};
use cw721::{Cw721ExecuteMsg, Cw721QueryMsg, Expiration as Cw721Expiration};
// use cw721_base::{ExecuteMsg as Cw721ExecuteMsgBase, InstantiateMsg as Cw721InstantiateMsg};

impl MarketplaceContract<'static> {
    pub fn validate_auction_config(&self, listing_config: &ListingConfig) -> bool {
        if listing_config.price.amount.is_zero() {
            return false;
        }
        // if start_time or end_time is not set, we don't need to check
        if listing_config.start_time.is_some()
            && listing_config.end_time.is_some()
            && listing_config.start_time.unwrap() >= listing_config.end_time.unwrap()
        {
            return false;
        }
        true
    }

    pub fn execute_list_nft(
        self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        contract_address: Addr,
        token_id: String,
        listing_config: ListingConfig,
    ) -> Result<Response, ContractError> {
        // check if user is the owner of the token
        let query_owner_msg = Cw721QueryMsg::OwnerOf {
            token_id: token_id.clone(),
            include_expired: Some(false),
        };
        let owner_response: StdResult<cw721::OwnerOfResponse> =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: contract_address.to_string(),
                msg: to_binary(&query_owner_msg)?,
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
        let query_approval_msg = Cw721QueryMsg::Approval {
            token_id: token_id.clone(),
            spender: env.contract.address.to_string(),
            include_expired: Some(true),
        };
        let approval_response: StdResult<cw721::ApprovalResponse> =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: contract_address.to_string(),
                msg: to_binary(&query_approval_msg)?,
            }));

        // check if approval is never expired
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

        if !self.validate_auction_config(&listing_config) {
            return Err(ContractError::CustomError {
                val: "Invalid auction config".to_string(),
            });
        }

        // add a nft to listings
        let listing = Listing {
            contract_address: contract_address.clone(),
            token_id: token_id.clone(),
            listing_config,
            seller: info.sender,
            buyer: None,
        };
        let listing_key = listing_key(&contract_address, &token_id);

        // we will override the listing if it already exists, so that we can update the listing config
        let new_listing = self.listings.update(
            deps.storage,
            listing_key,
            |_old| -> Result<Listing, ContractError> { Ok(listing) },
        )?;

        // println!("Listing: {:?}", _listing);
        let listing_config_str = serde_json::to_string(&new_listing.listing_config);
        match listing_config_str {
            Ok(listing_config_str) => Ok(Response::new()
                .add_attribute("method", "list_nft")
                .add_attribute("contract_address", new_listing.contract_address)
                .add_attribute("token_id", new_listing.token_id)
                .add_attribute("listing_config", listing_config_str)
                .add_attribute("seller", new_listing.seller.to_string())),
            Err(_) => Err(ContractError::CustomError {
                val: ("Listing Config Error".to_string()),
            }),
        }
    }

    pub fn execute_buy(
        self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        contract_address: Addr,
        token_id: String,
    ) -> Result<Response, ContractError> {
        // get the listing
        let listing_key = listing_key(&contract_address, &token_id);
        let mut listing = self.listings.load(deps.storage, listing_key.clone())?;

        // check if buyer is the same as seller
        if info.sender == listing.seller {
            return Err(ContractError::CustomError {
                val: ("Owner cannot buy".to_string()),
            });
        }

        listing.buyer = Some(info.sender.clone());

        // remove the listing
        self.listings.remove(deps.storage, listing_key)?;

        self.process_buy_fixed_price(deps, env, info, &listing)
    }

    fn process_buy_fixed_price(
        self,
        _deps: DepsMut,
        env: Env,
        info: MessageInfo,
        listing: &Listing,
    ) -> Result<Response, ContractError> {
        let price = listing.listing_config.price.clone();
        let start_time = listing.listing_config.start_time.clone();
        let end_time = listing.listing_config.end_time.clone();
        // check if current block is after start_time
        if start_time.is_some() && !start_time.unwrap().is_expired(&env.block) {
            return Err(ContractError::CustomError {
                val: ("Auction not started".to_string()),
            });
        }

        if end_time.is_some() && end_time.unwrap().is_expired(&env.block) {
            return Err(ContractError::CustomError {
                val: format!("Auction ended: {} {}", end_time.unwrap(), env.block.time),
            });
        }

        // check if enough funds
        if info.funds.is_empty() || info.funds[0] != price {
            return Err(ContractError::InsufficientFunds {});
        }

        // message to transfer nft to buyer
        let transfer_nft_msg = WasmMsg::Execute {
            contract_addr: listing.contract_address.to_string(),
            msg: to_binary(&Cw721ExecuteMsg::TransferNft {
                recipient: listing.buyer.clone().unwrap().into_string(),
                token_id: listing.token_id.clone(),
            })?,
            funds: vec![],
        };
        let mut res = Response::new().add_message(transfer_nft_msg);

        // TODO: payment
        // let payment = price;

        // let payment_messages = self.payment_with_royalty(
        //     &deps,
        //     &listing.contract_address,
        //     &listing.token_id,
        //     payment,
        //     &info.sender,
        //     &listing.seller,
        // );

        // for payment_message in payment_messages {
        //     res = res.add_message(payment_message);
        // }

        res = res
            .add_attribute("method", "buy")
            .add_attribute("contract_address", listing.contract_address.to_string())
            .add_attribute("token_id", listing.token_id.to_string())
            .add_attribute("buyer", info.sender);

        Ok(res)
    }

    pub fn execute_cancel(
        self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        contract_address: Addr,
        token_id: String,
    ) -> Result<Response, ContractError> {
        // find listing
        let listing_key = listing_key(&contract_address, &token_id);
        let listing = self.listings.load(deps.storage, listing_key.clone())?;

        // if a listing is not expired, only seller can cancel
        if (!listing.is_expired(&env.block)) && (listing.seller != info.sender) {
            return Err(ContractError::Unauthorized {});
        }

        // we will remove the cancelled listing
        self.listings.remove(deps.storage, listing_key)?;

        Ok(Response::new()
            .add_attribute("method", "cancel")
            .add_attribute("contract_address", contract_address)
            .add_attribute("token_id", token_id)
            .add_attribute("cancelled_at", env.block.time.to_string()))
    }

    pub fn execute_create_collection(
        self,
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _name: String,
        _symbol: String,
    ) -> Result<Response, ContractError> {
        // let msg = Cw721InstantiateMsg {
        //     name,
        //     symbol,
        //     minter: info.sender.to_string(),
        // };
        // let res = Response::new()
        //     .add_message(WasmMsg::Execute {
        //         contract_addr: info.sender.to_string(),
        //         msg: to_binary(&msg)?,
        //         funds: vec![],
        //     })
        //     .add_attribute("method", "create_collection")
        //     .add_attribute("name", name)
        //     .add_attribute("symbol", symbol)
        //     .add_attribute("minter", info.sender.to_string());
        // Ok(res)
        Ok(Response::new())
    }

    pub fn execute_mint_nft(
        self,
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _contract_address: Addr,
        _token_id: String,
        _token_uri: String,
    ) -> Result<Response, ContractError> {
        // let msg = Cw721ExecuteMsgBase::Mint {
        //     token_id: token_id.clone(),
        //     owner: info.sender.to_string(),
        //     token_uri: Some(token_uri.clone()),
        //     extension: None,
        // };
        // let res = Response::new()
        //     .add_message(WasmMsg::Execute {
        //         contract_addr: contract_address.to_string(),
        //         msg: to_binary(&msg)?,
        //         funds: vec![],
        //     })
        //     .add_attribute("method", "mint_nft")
        //     .add_attribute("contract_address", contract_address)
        //     .add_attribute("token_id", token_id)
        //     .add_attribute("token_uri", token_uri)
        //     .add_attribute("owner", info.sender.to_string());
        // Ok(res)
        Ok(Response::new())
    }
}
