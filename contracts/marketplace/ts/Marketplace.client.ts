/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.35.3.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult } from "@cosmjs/cosmwasm-stargate";
import { StdFee } from "@cosmjs/amino";
import { Addr, InstantiateMsg, ExecuteMsg, Expiration, Timestamp, Uint64, Uint128, ListingConfig, Coin, QueryMsg, Config, Listing, ListingsResponse } from "./Marketplace.types";
export interface MarketplaceReadOnlyInterface {
  contractAddress: string;
  config: () => Promise<Config>;
  listingsByContractAddress: ({
    contractAddress,
    limit,
    startAfter
  }: {
    contractAddress: string;
    limit?: number;
    startAfter?: string;
  }) => Promise<ListingsResponse>;
  listing: ({
    contractAddress,
    tokenId
  }: {
    contractAddress: string;
    tokenId: string;
  }) => Promise<Listing>;
}
export class MarketplaceQueryClient implements MarketplaceReadOnlyInterface {
  client: CosmWasmClient;
  contractAddress: string;

  constructor(client: CosmWasmClient, contractAddress: string) {
    this.client = client;
    this.contractAddress = contractAddress;
    this.config = this.config.bind(this);
    this.listingsByContractAddress = this.listingsByContractAddress.bind(this);
    this.listing = this.listing.bind(this);
  }

  config = async (): Promise<Config> => {
    return this.client.queryContractSmart(this.contractAddress, {
      config: {}
    });
  };
  listingsByContractAddress = async ({
    contractAddress,
    limit,
    startAfter
  }: {
    contractAddress: string;
    limit?: number;
    startAfter?: string;
  }): Promise<ListingsResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      listings_by_contract_address: {
        contract_address: contractAddress,
        limit,
        start_after: startAfter
      }
    });
  };
  listing = async ({
    contractAddress,
    tokenId
  }: {
    contractAddress: string;
    tokenId: string;
  }): Promise<Listing> => {
    return this.client.queryContractSmart(this.contractAddress, {
      listing: {
        contract_address: contractAddress,
        token_id: tokenId
      }
    });
  };
}
export interface MarketplaceInterface extends MarketplaceReadOnlyInterface {
  contractAddress: string;
  sender: string;
  listNft: ({
    contractAddress,
    listingConfig,
    tokenId
  }: {
    contractAddress: string;
    listingConfig: ListingConfig;
    tokenId: string;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  buy: ({
    contractAddress,
    tokenId
  }: {
    contractAddress: string;
    tokenId: string;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  cancel: ({
    contractAddress,
    tokenId
  }: {
    contractAddress: string;
    tokenId: string;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  createCollection: ({
    name,
    symbol
  }: {
    name: string;
    symbol: string;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  mintNft: ({
    contractAddress,
    tokenId,
    tokenUri
  }: {
    contractAddress: string;
    tokenId: string;
    tokenUri: string;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
}
export class MarketplaceClient extends MarketplaceQueryClient implements MarketplaceInterface {
  client: SigningCosmWasmClient;
  sender: string;
  contractAddress: string;

  constructor(client: SigningCosmWasmClient, sender: string, contractAddress: string) {
    super(client, contractAddress);
    this.client = client;
    this.sender = sender;
    this.contractAddress = contractAddress;
    this.listNft = this.listNft.bind(this);
    this.buy = this.buy.bind(this);
    this.cancel = this.cancel.bind(this);
    this.createCollection = this.createCollection.bind(this);
    this.mintNft = this.mintNft.bind(this);
  }

  listNft = async ({
    contractAddress,
    listingConfig,
    tokenId
  }: {
    contractAddress: string;
    listingConfig: ListingConfig;
    tokenId: string;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      list_nft: {
        contract_address: contractAddress,
        listing_config: listingConfig,
        token_id: tokenId
      }
    }, fee, memo, _funds);
  };
  buy = async ({
    contractAddress,
    tokenId
  }: {
    contractAddress: string;
    tokenId: string;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      buy: {
        contract_address: contractAddress,
        token_id: tokenId
      }
    }, fee, memo, _funds);
  };
  cancel = async ({
    contractAddress,
    tokenId
  }: {
    contractAddress: string;
    tokenId: string;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      cancel: {
        contract_address: contractAddress,
        token_id: tokenId
      }
    }, fee, memo, _funds);
  };
  createCollection = async ({
    name,
    symbol
  }: {
    name: string;
    symbol: string;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      create_collection: {
        name,
        symbol
      }
    }, fee, memo, _funds);
  };
  mintNft = async ({
    contractAddress,
    tokenId,
    tokenUri
  }: {
    contractAddress: string;
    tokenId: string;
    tokenUri: string;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      mint_nft: {
        contract_address: contractAddress,
        token_id: tokenId,
        token_uri: tokenUri
      }
    }, fee, memo, _funds);
  };
}