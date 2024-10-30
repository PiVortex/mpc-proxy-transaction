use hex::FromHex;
use near_sdk::json_types::U64;
use near_sdk::{env, near, serde_json, Gas, NearToken, PromiseError, PromiseOrValue};
use omni_transaction::near::near_transaction::NearTransaction;
use omni_transaction::near::types::{
    AccessKey, AccessKeyPermission, Action, AddKeyAction, Secp256K1Signature, Signature,
    U64 as OmniU64,
};
use omni_transaction::near::utils::PublicKeyStrExt;
use omni_transaction::transaction_builder::{TransactionBuilder, TxBuilder};
use omni_transaction::types::NEAR;
use sha2::{Digest, Sha256};

const SIGN_CALLBACK_GAS: Gas = Gas::from_tgas(30);

pub mod signer;
pub use crate::signer::*;

#[near(serializers = [json])]
pub struct FuncInput {
    target_account: String,
    target_public_key: String,
    nonce: U64,
    block_hash: String,
    new_public_key_to_add: String,
    mpc_deposit: NearToken,
}

#[derive(Default)]
#[near(contract_state)]
pub struct Contract {}

#[near]
impl Contract {
    pub fn proxy_send_near(&mut self, input: FuncInput) -> PromiseOrValue<String> {
        // Prepare add key action
        let add_key_action = Action::AddKey(Box::new(AddKeyAction {
            public_key: input.new_public_key_to_add.to_public_key().unwrap(),
            access_key: AccessKey {
                nonce: OmniU64(0),
                permission: AccessKeyPermission::FullAccess,
            },
        }));

        // Add action to a vector of actions
        let actions = vec![add_key_action];

        // Build NearTransaction
        let near_tx = TransactionBuilder::new::<NEAR>()
            .signer_id(input.target_account.to_string())
            .signer_public_key(input.target_public_key.to_public_key().unwrap())
            .nonce(input.nonce.0)
            .receiver_id(input.target_account.to_string())
            .block_hash(input.block_hash.to_block_hash().unwrap())
            .actions(actions)
            .build();

        // Get the transaction payload and hash it
        let payload = near_tx.build_for_signing();
        let hashed_payload = hash_payload(&payload);

        // Serialize NearTransaction into a string to pass into callback
        let serialized_tx = serde_json::to_string(&near_tx)
            .unwrap_or_else(|e| panic!("Failed to serialize NearTransaction: {:?}", e));

        // Call MPC
        PromiseOrValue::Promise(
            ext_signer::ext("v1.signer-prod.testnet".parse().unwrap())
                .with_attached_deposit(input.mpc_deposit)
                .sign(SignRequest::new(
                    hashed_payload
                        .try_into()
                        .unwrap_or_else(|e| panic!("Failed to convert payload {:?}", e)),
                    input.target_account.to_string(),
                    0,
                ))
                .then(
                    Self::ext(env::current_account_id())
                        .with_static_gas(SIGN_CALLBACK_GAS)
                        .with_unused_gas_weight(0)
                        .sign_callback(serialized_tx),
                ),
        )
    }

    #[private]
    pub fn sign_callback(
        &self,
        #[callback_result] result: Result<SignResult, PromiseError>,
        serialized_tx: String,
    ) -> Vec<u8> {
        if let Ok(sign_result) = result {
            // Reconstruct signature
            let big_r = &sign_result.big_r.affine_point;
            let s = &sign_result.s.scalar;

            let r = &big_r[2..];
            let end = &big_r[..2];

            let r_bytes = Vec::from_hex(r).expect("Invalid hex in r");
            let s_bytes = Vec::from_hex(s).expect("Invalid hex in s");
            let end_bytes = Vec::from_hex(end).expect("Invalid hex in end");

            let mut signature_bytes = [0u8; 65];
            signature_bytes[..32].copy_from_slice(&r_bytes);
            signature_bytes[32..64].copy_from_slice(&s_bytes);
            signature_bytes[64] = end_bytes[0];

            let omni_signature = Signature::SECP256K1(Secp256K1Signature(signature_bytes));

            // Deserialize NearTransaction
            let near_tx = serde_json::from_str::<NearTransaction>(&serialized_tx)
                .unwrap_or_else(|e| panic!("Failed to deserialize NearTransaction: {:?}", e));

            // Add signature to transaction
            let near_tx_signed = near_tx.build_with_signature(omni_signature);

            // Return signed transaction
            near_tx_signed
        } else {
            panic!("Callback failed");
        }
    }
}

// Function to hash payload
pub fn hash_payload(payload: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(payload);
    let result = hasher.finalize();
    result.into()
}
