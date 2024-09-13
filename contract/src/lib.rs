use near_sdk::{near, PromiseOrValue, log, NearToken, env, PromiseError, Gas};
use near_sdk::json_types::{U128, U64};
use omni_transaction::near::types::{Action, TransferAction, Signature, ED25519Signature};
use omni_transaction::transaction_builder::TransactionBuilder;
use omni_transaction::transaction_builder::TxBuilder;
use omni_transaction::types::NEAR;
use omni_transaction::near::utils::PublicKeyStrExt;
use omni_transaction::near::near_transaction::NearTransaction;

pub mod signer;
pub use crate::signer::*;

#[near(serializers = [json])]
pub struct FuncInput {
    target_account: String,
    target_public_key: String,
    nonce: U64,
    block_hash: String,
}

#[derive(Default)]
#[near(contract_state)]
pub struct Contract {
    last_tx: Option<NearTransaction>
}

const SIGN_CALLBACK_GAS: Gas = Gas::from_tgas(10);

#[near]
impl Contract {
    pub fn proxy_send_near(&mut self, input: FuncInput) -> PromiseOrValue<String> {

        let receiver_id = "pivortex.testnet";
        let transfer_action = Action::Transfer(TransferAction { deposit: U128(1).0.into() });
        let actions = vec![transfer_action];

        let near_tx = TransactionBuilder::new::<NEAR>()
            .signer_id(input.target_account.to_string())
            .signer_public_key(input.target_public_key.to_public_key().unwrap())
            .nonce(input.nonce.0)
            .receiver_id(receiver_id.to_string())
            .block_hash(input.block_hash.to_block_hash().unwrap())
            .actions(actions)
            .build();

        let payload = near_tx.build_for_signing();

        self.last_tx = Some(near_tx);

        let payload_slice: [u8; 32] = payload[0..32]
            .try_into()
            .expect("Something went wrong");

        // Call MPC
        PromiseOrValue::Promise(
            ext_signer::ext("v1.signer-prod.testnet".parse().unwrap())
                .with_attached_deposit(NearToken::from_millinear(500))
                .sign(SignRequest::new(
                    payload_slice.try_into().unwrap_or_else(|e| panic!("Failed to convert payload {:?}", e)),
                    input.target_account.to_string(),
                    0,
                )) .then(
                    Self::ext(env::current_account_id())
                        .with_static_gas(SIGN_CALLBACK_GAS)
                        .with_unused_gas_weight(0)
                        .sign_callback(),
                ),
        )
    }

    // pub fn get_last_tx(&self) -> &Option<NearTransaction> {
    //     &self.last_tx
    // }

    #[private]
    pub fn sign_callback(
        &self,
        #[callback_result] result: Result<SignResult, PromiseError>,
    ) -> String {
        // Build for signing

        if let Ok(sign_result) = result {
            let r = sign_result.big_r.affine_point;
            let s = sign_result.s.scalar;

            
        let omni_signature = Signature::ED25519(ED25519Signature {
            r: signature_bytes[..32].try_into().unwrap(),
            s: signature_bytes[32..].try_into().unwrap(),
        });

        // Build the signed transaction
        let near_tx_signed = near_tx.build_with_signature(omni_signature);

            return "something".to_string();

        } else {
            panic!("Callback failed");
        }
    
    }
    
    
}

