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
const deployerWallet = await DirectSecp256k1HdWallet.fromMnemonic(
    (await readFile("./sample.mnemonic.key")).toString(),
    {
        prefix: prefix,
    }
);
// config gas price
const gasPrice = GasPrice.fromString(`0.025${denom}`);
const deployerClient = await SigningCosmWasmClient.connectWithSigner(RPCEndpoint, deployerWallet, {gasPrice});
const deployerAccount = (await deployerWallet.getAccounts())[0];

// ********************
// EXECUTE CONTRACT
// ********************
// prepare execute message
const addNativeTokenDecimalsMsg = {
    "add_native_token_decimals": {
        "denom": "uaura",
        "decimals": 6,
    }
}

// Execute contract
let executeResponse = await deployerClient.execute(
    deployerAccount.address,
    contractAddress,
    addNativeTokenDecimalsMsg,
    "auto",
    "add new native token decimals",
    [coin(1, "uaura")],
);

// print out the transaction's info
console.log("TransactionHash: ", executeResponse.transactionHash);
console.log("GasWanted / gasUsed: ", executeResponse.gasWanted, " / ", executeResponse.gasUsed);