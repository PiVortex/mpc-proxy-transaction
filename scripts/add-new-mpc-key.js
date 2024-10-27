const { connect, keyStores, providers } = require('near-api-js');
const homedir = require("os").homedir();
const path = require("path");
const { deriveKey } = require('./derive-key-util');

const CREDENTIALS_DIR = ".near-credentials";
const credentialsPath = path.join(homedir, CREDENTIALS_DIR);
const myKeyStore = new keyStores.UnencryptedFileSystemKeyStore(credentialsPath);

const connectionConfig = {
  networkId: "testnet",
  keyStore: myKeyStore,
  nodeUrl: "https://rpc.testnet.near.org",
  walletUrl: "https://testnet.mynearwallet.com/",
  helperUrl: "https://helper.testnet.near.org",
  explorerUrl: "https://testnet.nearblocks.io",
};

const PROXY_CONTRACT = "proxy-1.testnet"

async function main(targetAccountId) {
    const near = await connect(connectionConfig);
    const target_account = await near.account(targetAccountId);

    // Derive public key for the MPC to use
    // This is a secp256k1 key as the MPC contract only uses this curve right now
    // Predecessor to MPC contract is the proxy contract
    // Path is the account Id of the account we are adding the new key to
    const publicKey = await deriveKey(PROXY_CONTRACT, targetAccountId);
        
    // Add the public key to the target account
    await target_account.addKey(publicKey);
}

main("just-some-account.testnet")