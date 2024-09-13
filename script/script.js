const { connect, keyStores, providers } = require('near-api-js');
const homedir = require("os").homedir();
const path = require("path");
const { deriveKey } = require('./kdf');

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
    publicKey = await deriveKey(PROXY_CONTRACT, targetAccountId);
    
    // Add the public key to the target account
    // await account.addKey(publicKey);

    // Create input 

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

    // console.log(outcome);

    result = providers.getTransactionLastResult(outcome);
    console.log(result);
    // let {big_r, s, recovery_id} = result;
    // let r = big_r.affine_point.slice(2);
    // s = s.scalar;

    // Reconstruct signature
    
    // const message = encodeTransaction(input)



    // const signedTx = new SignedTransaction({
    //   transaction,
    //   signature: new Signature({ keyType, data: signature.signature })
    // });

    // Send transaction
    // providers.sendTransaction(SignedTx);
}



main("account-with-eth-key.testnet");