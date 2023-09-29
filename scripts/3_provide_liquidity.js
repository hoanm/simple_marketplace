const chainConfig = require('./config/chain').defaultChain;

const fs = require('fs');

const { SigningCosmWasmClient } = require('@cosmjs/cosmwasm-stargate');
const { DirectSecp256k1HdWallet, coin } = require('@cosmjs/proto-signing');
const { calculateFee, GasPrice } = require('@cosmjs/stargate');

// wasm folder
const wasmFolder = `${__dirname}/../artifacts`;

// gas price
const gasPrice = GasPrice.fromString(`0.025${chainConfig.denom}`);
// tester and deployer info
let testerWallet, testerClient, testerAccount;
let deployerWallet, deployerClient, deployerAccount;


/// @dev Store the contract source code on chain
/// @param `wasm_name` - The name of the wasm file
/// @return `storeCodeResponse` - The response of the store code transaction
async function store_contract(wasm_name) {
    const uploadFee = calculateFee(2600000, gasPrice);
    const contractCode = fs.readFileSync(`${wasmFolder}/${wasm_name}.wasm`);
    
    console.log("Uploading contract code...");
    const storeCodeResponse = await deployerClient.upload(deployerAccount.address, contractCode, uploadFee, 'Upload nft_launchapad contract code');
    
    console.log("  transactionHash: ", storeCodeResponse.transactionHash);
    console.log("  codeId: ", storeCodeResponse.codeId);
    console.log("  gasWanted / gasUsed: ", storeCodeResponse.gasWanted, " / ", storeCodeResponse.gasUsed);

    return storeCodeResponse;
}

/// @dev Instantiate contract base on the code id and instantiate message of contract
/// @param `_codeID` - The code id of the contract
/// @param `instantiateMsg` - The instantiate message of the contract
/// @return `instantiateResponse` - The response of the instantiate transaction
async function instantiate(contract_code_id, instantiateMsg) {
    console.log("Instantiating contract...");

    //Instantiate the contract
    const instantiateResponse = await deployerClient.instantiate(
        deployerAccount.address,
        Number(contract_code_id),
        instantiateMsg,
        "instantiation contract",
        "auto",
    );
    console.log("  transactionHash: ", instantiateResponse.transactionHash);
    console.log("  contractAddress: ", instantiateResponse.contractAddress);
    console.log("  gasWanted / gasUsed: ", instantiateResponse.gasWanted, " / ", instantiateResponse.gasUsed);
    
    return instantiateResponse;
}

/// @dev Execute a message to the contract
/// @param `userClient` - The client of the user who execute the message
/// @param `userAccount` -  The account of the user who execute the message
/// @param `contract` - The address of the contract
/// @param `executeMsg` - The message that will be executed
/// @return `executeResponse` - The response of the execute transaction
async function execute(userClient, userAccount, contract, executeMsg, native_amount = 0, native_denom = chainConfig.denom) {
    console.log("Executing message to contract...");

    const memo = "execute a message";

    let executeResponse;

    // if the native amount is not 0, then send the native token to the contract
    if (native_amount != 0) {
        executeResponse = await userClient.execute(
            userAccount.address,
            contract,
            executeMsg,
            "auto",
            memo,
            [coin(native_amount, native_denom)],
        );
    } else {
        executeResponse = await userClient.execute(
            userAccount.address,
            contract,
            executeMsg,
            "auto",
            memo,
        );
    }
    
    
    console.log("  transactionHash: ", executeResponse.transactionHash);
    console.log("  gasWanted / gasUsed: ", executeResponse.gasWanted, " / ", executeResponse.gasUsed);

    return executeResponse;
}

/// @dev Query information from the contract
/// @param `userClient` - The client of the user who execute the message
/// @param `contract` - The address of the contract
/// @param `queryMsg` - The message that will be executed
/// @return `queryResponse` - The response of the query
async function query(userClient, contract, queryMsg) {
    console.log("Querying contract...");

    const queryResponse = await userClient.queryContractSmart(contract, queryMsg);

    console.log("  Querying successful");
    
    return queryResponse;
}

async function main() {
    // ***************************
    // SETUP INFORMATION FOR USERS
    // ***************************
    // connect deployer wallet to chain and get admin account
    deployerWallet = await DirectSecp256k1HdWallet.fromMnemonic(
        "senior merit prison cruise rice cluster upset ignore amazing dust unfold motor",
        {
            prefix: chainConfig.prefix
        }
    );
    deployerClient = await SigningCosmWasmClient.connectWithSigner(chainConfig.rpcEndpoint, deployerWallet, {gasPrice});
    deployerAccount = (await deployerWallet.getAccounts())[0];

    // // create message to transfer cw20 token
    // const transferMsg = {
    //     "provide_liquidity": {
    //         "assets": [
    //           {
    //               "info": {
    //                  "native_token": {
    //                     "denom": "ibc/035BDC396AA81E38271D2FA5E4799AE159044B90BCF02CCA218EB364829C869E"
    //                 }
    //              },
    //               "amount": "7003000000"
    //           },
    //           {
    //               "info": {
    //                  "token": {
    //                     "contract_addr": "aura1svdtvvut8q9w60d9cvw8gzr22vae28z3dpupzj5hnzhwmf5qfhcq69mgz8"
    //                 }
    //              },
    //               "amount": "9415719396384"
    //           }
    //        ]
    //     }
    // };

    // create message to transfer cw20 token
    const transferMsg = {
        "provide_liquidity": {
            "assets": [
              {
                  "info": {
                     "native_token": {
                        "denom": "ueaura"
                    }
                 },
                  "amount": "1550000"
              },
              {
                  "info": {
                     "native_token": {
                        "denom": "ibc/035BDC396AA81E38271D2FA5E4799AE159044B90BCF02CCA218EB364829C869E"
                    }
                 },
                  "amount": "23493700000"
              }
           ]
        }
    };

    executeResponse = await deployerClient.execute(
        deployerAccount.address,
        "aura1faq2s9ze2jqdz4pr8fhhny4zma903tpr97nnw27jc95cmpt9kvgq33qjse",
        transferMsg,
        "auto",
        "provide liquidity",
        [coin("23493700000", "ibc/035BDC396AA81E38271D2FA5E4799AE159044B90BCF02CCA218EB364829C869E"), coin("1550000", "ueaura")],
    );

    // // execute the transfer message
    // const transferResponse = await execute(
    //     deployerClient, 
    //     deployerAccount, 
    //     "aura1faq2s9ze2jqdz4pr8fhhny4zma903tpr97nnw27jc95cmpt9kvgq33qjse", 
    //     transferMsg,
    //     "8219832024000"
    // );
}

const myArgs = process.argv.slice(2);
// if (myArgs.length != 1) {
//     console.log("Usage: node 0_launchpad_setup.js <wasm_contract_name>");
//     process.exit(1);
// }
main();
