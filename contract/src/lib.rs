use near_sdk::{near, PromiseOrValue, log, NearToken, env, PromiseError, Gas};
use near_sdk::json_types::{U128, U64};
use omni_transaction::near::types::{Action, TransferAction, Signature, Secp256K1Signature};
use omni_transaction::transaction_builder::TransactionBuilder;
use omni_transaction::transaction_builder::TxBuilder;
use omni_transaction::types::NEAR;
use omni_transaction::near::utils::PublicKeyStrExt;
use omni_transaction::near::near_transaction::NearTransaction;

const SIGN_CALLBACK_GAS: Gas = Gas::from_tgas(10);

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
    ) -> Vec<u8> {
        // Build for signing

        if let Ok(sign_result) = result {
            let r = &sign_result.big_r.affine_point;
            let s = &sign_result.s.scalar;

            // Get signature like secp256k1::soandso and then i can use .to_signature on it 

            let r_bytes = hex::decode(r).expect("Failed to decode r from hex");
            let s_bytes = hex::decode(s).expect("Failed to decode s from hex");
            
            log!("r: {:?}", r_bytes.len());
            log!("s: {:?}", s_bytes.len());

            let mut signature_bytes = [0u8; 65];

            // Step 4: Copy r into the first 33 bytes
            signature_bytes[..33].copy_from_slice(&r_bytes);
        
            // Step 5: Copy s into the next 32 bytes
            signature_bytes[33..65].copy_from_slice(&s_bytes);

            let omni_signature = Signature::SECP256K1(Secp256K1Signature (signature_bytes));

            let near_tx_signed = self.last_tx.as_ref().unwrap().build_with_signature(omni_signature);

            return near_tx_signed;
        } else {
            panic!("Callback failed");
        }
    }
    
    
}

