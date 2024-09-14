const { connect, keyStores, providers, transactions, utils, publicKey } = require('near-api-js');
const homedir = require("os").homedir();
const path = require("path");
const { deriveKey } = require('./kdf');
const fs = require('fs');
const { PublicKey } = require('near-api-js/lib/utils');
const sha256 = require('js-sha256');

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

//path is account Id, account for derviation is proxy contract
async function main(targetAccountId) {
    const near = await connect(connectionConfig);
    const account = await near.account(targetAccountId);

    // Derive public key for the MPC to use
    const publicKey = await deriveKey(PROXY_CONTRACT, targetAccountId);
        
    // Add the public key to the target account
    // await account.addKey(publicKey);

    // Get nonce
    const accessKey = await near.connection.provider.query({
      request_type: 'view_access_key',
      account_id: targetAccountId,
      public_key: publicKey,
      finality: 'optimistic'
    });
    const nonce = accessKey.nonce;

    // Get block hash
    const block = await near.connection.provider.block({
      finality: "final",
    });
    const blockHash = block.header.hash;

    // Prepare input
    const input = {
        target_account: targetAccountId,
        target_public_key: publicKey,
        nonce: (nonce + 1).toString(),
        block_hash: blockHash,
    }

    // Call the proxy contract to get signature 
    const outcome = await account.functionCall({
        contractId: PROXY_CONTRACT,
        methodName: "proxy_send_near",
        args: {
            input,
        },
        gas: "300000000000000",
        attachedDeposit: 0,
    })

    // Get signature
    result = providers.getTransactionLastResult(outcome);

    const res = new Uint8Array(result);
    // fs.writeFileSync('data.bin', Buffer.from(res)); // Choose to save in file so don't have to call contract again

    // Read signedTransaction binary from file and decode
    // const rawBytes = fs.readFileSync('data.bin'); 
    // const res = new Uint8Array(rawBytes);

    // Decode signedTransaction
    const signedTransaction = transactions.SignedTransaction.decode(res);
    console.log(signedTransaction);
    
    // Get public key object
    const key_obj = PublicKey.fromString(publicKey);

    // Get txHash from transaction
    const serializedTx = utils.serialize.serialize(
      transactions.SCHEMA.Transaction,
      signedTransaction.transaction
    );
    const txHash = new Uint8Array(sha256.sha256.array(serializedTx));

    // Get signature 
    const signature = new Uint8Array(signedTransaction.signature.secp256k1Signature.data);
    console.log(signature);

    // Check if signature is valid
    const isValid = key_obj.verify(txHash, signature);
    console.log(isValid); // Signature is not valid
}

main("account-with-eth-key.testnet");