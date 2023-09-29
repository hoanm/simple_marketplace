const fs = require('fs/promises');
const { SigningCosmWasmClient } = require("@cosmjs/cosmwasm-stargate");
const { DirectSecp256k1HdWallet } = require("@cosmjs/proto-signing");
const { GasPrice } = require("@cosmjs/stargate");

// config chain
const RPCEndpoint = "https://rpc.euphoria.aura.network";
const prefix = "aura";
const denom = "uaura";

// you must provide the address of the contract you want to execute here
const contractAddress = "aura100000000000000000000000000000000000000000000000HaloFactory";

// ***************************
// SETUP INFORMATION FOR USERS
// ***************************
// connect deployer wallet to chain and get admin account
const userWallet = await DirectSecp256k1HdWallet.fromMnemonic(
    (await readFile("./sample.mnemonic.key")).toString(),
    {
        prefix: prefix,
    }
);
// config gas price
const gasPrice = GasPrice.fromString(`0.025${denom}`);
const userClient = await SigningCosmWasmClient.connectWithSigner(RPCEndpoint, userWallet, {gasPrice});

// ********************
// QUERY CONTRACT
// ********************
// prepare query message
const nativeTokenDecimalsMsg = {
    "native_token_decimals": {
        "denom": "uaura",
    },
}

// Query contract
const queryResponse = await userClient.queryContractSmart(contractAddress, nativeTokenDecimalsMsg);

// print out the query response
console.log(queryResponse);