import dotenv from "dotenv";
import { SSL_OP_SSLEAY_080_CLIENT_DH_BUG } from "node:constants";
dotenv.config();
import * as readline from 'node:readline';
import { promisify } from 'util';
import { ajay_wallet, ClubStakingContractPath, TokenPath, PrismIntegrationPath, PrismForguePath, liquidity_wallet, marketing_wallet, MintingContractPath, mintInitMessage, mint_wallet, nitin_wallet, sameer_wallet, team_wallet, terraClient, treasury_wallet } from './constants.js';
import { primeAccountsWithFunds } from "./primeCustomAccounts.js";
import {
  executeContract, getGasUsed, instantiateContract, queryContract, readArtifact, storeCode,
  writeArtifact, queryBankUusd, queryContractInfo, readDistantArtifact,
  queryTokenBalance
} from './utils.js';

async function uploadPrismIntegration() {
  console.log("Uploading Prism Integration...");
  let contractId = await storeCode(mint_wallet, PrismIntegrationPath); // Getting the contract id from local terra
  console.log(`Prism Integration Contract ID: ${contractId}`);
  return contractId;
}

async function uploadPrismForgue() {
  console.log("Uploading Prism Forgue...");
  let contractId = await storeCode(mint_wallet, PrismForguePath); // Getting the contract id from local terra
  console.log(`Prism Forgue Contract ID: ${contractId}`);
  return contractId;
}


async function uploadToken() {
  console.log("Uploading Prism Forgue...");
  let contractId = await storeCode(mint_wallet, TokenPath); // Getting the contract id from local terra
  console.log(`Prism Forgue Contract ID: ${contractId}`);
  return contractId;
}

async function instantiatePrismIntegration(prismIntegrationId) {

  console.log("Instantiating Prism Integration...");

  let prismInitMessage = {
    owner: mint_wallet.key.accAddress,
    denom: "uusd"
  }

  console.log(JSON.stringify(prismInitMessage, null, 2));
  let result = await instantiateContract(mint_wallet, prismIntegrationId, prismInitMessage);
  let contractAddresses = result.logs[0].events[0].attributes.filter(element => element.key == 'contract_address').map(x => x.value);
  console.log(`Prism Forge Contract Address: ${contractAddresses}`);
  return contractAddresses;
}

async function instantiateToken(tokenId) {

  console.log("Instantiating Token Contract...");

  let tokenInitMessage = {
    name: "Fury",
    symbol: "Fury",
    decimals: 6,
    initial_balances: [],
    mint: {
      minter: mint_wallet.key.accAddress
    }
  }

  console.log(JSON.stringify(tokenInitMessage, null, 2));
  let result = await instantiateContract(mint_wallet, tokenId, tokenInitMessage);
  let contractAddresses = result.logs[0].events[0].attributes.filter(element => element.key == 'contract_address').map(x => x.value);
  console.log(`Token Contract Address: ${contractAddresses}`);
  return contractAddresses;
}

async function instantiatePrismForge(prismIntegrationId, tokenAddress) {
  console.log("Instantiating Prism Integration...");

  let prismInitMessage = {
    operator: mint_wallet.key.accAddress,
    receiver: nitin_wallet.key.accAddress,
    token: tokenAddress,
    base_denom: "uusd",
    host_portion: "0.1",
    host_portion_receiver: nitin_wallet.key.accAddress
  }
  console.log(JSON.stringify(prismInitMessage, null, 2));
  let result = await instantiateContract(mint_wallet, prismIntegrationId, prismInitMessage);
  let contractAddresses = result.logs[0].events[0].attributes.filter(element => element.key == 'contract_address').map(x => x.value);
  console.log(`Prism Integration Contract Address: ${contractAddresses}`);
  return contractAddresses;
}



async function setPrismForgeAddress(prismIntegrationAddress, prismForgeAddress) {
  let setPrimAddressRequest = {
    set_prism_address: {
      address: prismForgeAddress
    }
  };
  console.log(setPrimAddressRequest);
  let wsfacResponse = await executeContract(mint_wallet, prismIntegrationAddress, setPrimAddressRequest, {});
  console.log("set prism contract address transaction hash = " + wsfacResponse['txhash']);
}



async function getPrismForgueAddress(prismIntegrationAddress) {
  let coResponse = await queryContract(prismIntegrationAddress, {
    get_prism_address: {}
  });
  console.log(coResponse);
  return coResponse;
}


async function getStateInfo(prismIntegrationAddress) {
  let coResponse = await queryContract(prismIntegrationAddress, {
    get_state_info: {}
  });
  console.log(coResponse);
  return coResponse;
}

async function prismDeposit(prismIntegrationAddress) {
  let deposit = {
    deposit: {}
  };
  let wsfacResponse = await executeContract(mint_wallet, prismIntegrationAddress, deposit, { 'uusd': Number(1000000) });
  console.log("transaction hash = " + wsfacResponse['txhash']);
}

async function postInitialize(prismForgeAddress) {

  let postInit = {
    post_initialize: {
      launch_config: {
        amount: "100000",
        phase1_start: Date.parse("April 12, 2022 00:41:00") / 1000,
        phase2_start: Date.parse("April 12, 2022 01:00:00") / 1000,
        phase2_end: Date.parse("April 12, 2022 02:00:00") / 1000,
        phase2_slot_period: Date.parse("April 12, 2022 02:00:00") / 1000 - Date.parse("April 12, 2022 01:00:00") / 1000
      }
    }
  };

  console.log(postInit)
  let wsfacResponse = await executeContract(mint_wallet, prismForgeAddress, postInit, {});
  console.log("set prism contract address transaction hash = " + wsfacResponse['txhash']);
}



async function getDepositInfo(prismIntegrationAddress) {
  let coResponse = await queryContract(prismIntegrationAddress, {
    deposit_info: { address: mint_wallet.key.accAddress }
  });
  console.log(coResponse);
  return coResponse;
}

async function mint(tokenAddress) {
  let mint = {
    mint: {
      recipient: mint_wallet.key.accAddress,
      amount: "3000000"
    }

  };

  let wsfacResponse = await executeContract(mint_wallet, tokenAddress, mint, {});
  console.log("token mint transaction hash = " + wsfacResponse['txhash']);
}


async function getBalance(tokenAddress) {
  let coResponse = await queryContract(tokenAddress, {
    balance: { address: mint_wallet.key.accAddress }
  });
  console.log(coResponse);
  return coResponse;
}

async function increaseAllowance(tokenAddress, prismForgeAddress) {
  let allowance = {
    increase_allowance: {
      spender: prismForgeAddress,
      amount: "3000000"
    }
  };

  console.log(allowance);

  let wsfacResponse = await executeContract(mint_wallet, tokenAddress, allowance, {});
  console.log("set prism contract address transaction hash = " + wsfacResponse['txhash']);
}



async function main() {
  const prismIntegrationId = 62243;
  const prismForgeId = 62242;
  const tokenId = 62265;

  const prismIntegrationAddress = "terra1qp9q423ak0fj8wxfvj9k8xyk489mkaqt06qrz0";
  const tokenContractAddress = "terra1fm34sn99kq70pguxxcnyuxk7mx4hajh8qkzs9d";
  const prismForgeAddress = "terra1kyzfnujlyx6mq4xccdtrvzparsdj55n8xqvme8";

  //Uploading 3 contracts
  //let prismIntegrationId = await uploadPrismIntegration();
  //let prismForgeId = await uploadPrismForgue();
  //let tokenId = await uploadToken();


  //Instantiating contract
  //let prismIntegrationAddress = await instantiatePrismIntegration(prismIntegrationId);
  //let tokenContractAddress = await instantiateToken(tokenId);
  //let prismForgAddress = await instantiatePrismForge(prismForgeId, tokenContractAddress);


  await setPrismForgeAddress(prismIntegrationAddress, prismForgeAddress);
  await getPrismForgueAddress(prismIntegrationAddress);
  await getStateInfo(prismIntegrationAddress);


  await mint(tokenContractAddress);
  await getBalance(tokenContractAddress);
  await increaseAllowance(tokenContractAddress, prismForgeAddress);
  await postInitialize(prismForgeAddress);

  await prismDeposit(prismIntegrationAddress)
  await getDepositInfo(prismForgeAddress);

}

main();