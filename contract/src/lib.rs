use near_sdk::{near, PromiseOrValue, log, NearToken};
use near_sdk::json_types::{U128, U64};
use omni_transaction::near::types::{Action, TransferAction};
use omni_transaction::transaction_builder::TransactionBuilder;
use omni_transaction::transaction_builder::TxBuilder;
use omni_transaction::types::NEAR;
use omni_transaction::near::utils::PublicKeyStrExt;

pub mod signer;
pub use crate::signer::*;

#[near(serializers = [json])]
pub struct FuncInput {
    target_account: String,
    target_public_key: String,
    nonce: U64,
    block_hash: String,
}

#[near(contract_state)]
pub struct Contract;

#[near]
impl Contract {
    pub fn proxy_send_near(input: FuncInput) -> PromiseOrValue<String> {

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

        log!("Transaction: {:?}", near_tx);

        let payload = near_tx.build_for_signing();

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
                ))
        )
    }
}

