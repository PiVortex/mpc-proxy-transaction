## mpc-proxy-transaction

> [!WARNING]  
> This is a work in progress and should not be used.

This is a simple example of adding a secp256k1 key controlled by the MPC contract to an existing NEAR account then using that key to add a new full access ed25519 key to the account.

Currently by adding the dervied key to an account anyone can get a private key for that account. To make this contract good it should:
- Implement a threshold of accounts that need to sign a transaction to add a new key (2 of 3, 3 of 5, etc).
- Implement a ZKP verifier to prove that a user owns an email address (zk email, google auth) or knows a password of certain complexity.