use near_sdk::{ext_contract, near, PromiseOrValue};

#[near(serializers = [json])]
pub struct SignRequest {
    pub payload: [u8; 32],
    pub path: String,
    pub key_version: u32,
}

impl SignRequest {
    pub fn new(payload: [u8; 32], path: String, key_version: u32) -> Self {
        Self {
            payload,
            path,
            key_version,
        }
    }
}

#[near(serializers = [json])]
pub struct SignResult {
    pub big_r: String,
    pub s: String,
}


#[ext_contract(ext_signer)]
pub trait SignerInterface {
    fn sign(&mut self, request: SignRequest) -> PromiseOrValue<SignResult>;
}