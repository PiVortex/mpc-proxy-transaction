const { connect, keyStores, providers, KeyPair } = require('near-api-js');
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
    const relayer_account = await near.account("pivortex.testnet");

    // Generate a new ed25519 key pair to add to the target account
    const newKeyPair = KeyPair.fromRandom('ed25519'); // Create key pair
    const ed25519PublicKey = newKeyPair.publicKey.toString();
    const ed25519privateKey = newKeyPair.toString();
    
    console.log("New public Key: ", ed25519PublicKey);
    console.log("New private Key: ", ed25519privateKey);

    // Get the public on the target account that the MPC has control over
    const secp256k1PublicKey = await deriveKey(PROXY_CONTRACT, targetAccountId);
        
    // Get the nonce of the keys 
    const accessKey = await near.connection.provider.query({
      request_type: 'view_access_key',
      account_id: targetAccountId,
      public_key: secp256k1PublicKey,
      finality: 'optimistic'
    });
    const nonce = accessKey.nonce;

    // Get recent block hash
    const block = await near.connection.provider.block({
      finality: "final",
    });
    const blockHash = block.header.hash;

    // Prepare input
    const input = {
        target_account: targetAccountId,
        target_public_key: secp256k1PublicKey,
        nonce: (nonce + 1).toString(),
        block_hash: blockHash,
        new_public_key_to_add: ed25519PublicKey,
        mpc_deposit: "500000000000000000000000" // 0.5 NEAR will be plenty 
    }

    // Call the proxy contract to get signed add new key transaction
    const outcome = await relayer_account.functionCall({
        contractId: PROXY_CONTRACT,
        methodName: "proxy_send_near",
        args: {
            input,
        },
        gas: "300000000000000",
        attachedDeposit: 0,
    })

    result = providers.getTransactionLastResult(outcome);
    const signedSerializedTx = new Uint8Array(result);

    const send_result = await near.connection.provider.sendJsonRpc("broadcast_tx_commit", [
      Buffer.from(signedSerializedTx).toString("base64"),
    ]);

  console.log(send_result);
}

main("just-some-account.testnet");