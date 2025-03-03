use base64::{Engine, prelude::BASE64_STANDARD};
use eyre::Result;
use semaphore_rs::{Field, identity::Identity, protocol::Proof};
use world_chain_builder_pbh::{
    date_marker::DateMarker,
    external_nullifier::{EncodedExternalNullifier, ExternalNullifier},
    payload::PBHPayload,
};

pub struct WorldID {
    pub identity: Identity,
}

impl WorldID {
    pub fn new(secret: &str) -> Result<Self> {
        let decoded = BASE64_STANDARD.decode(secret)?;

        if decoded.len() != 64 {
            return Err(eyre::eyre!("Invalid identity length"));
        }

        let trapdoor = &decoded[..32];
        let nullifier = &decoded[32..];

        let identity = Identity {
            trapdoor: Field::from_be_slice(trapdoor),
            nullifier: Field::from_be_slice(nullifier),
        };

        Ok(Self { identity })
    }

    pub fn identity(&self) -> &Identity {
        &self.identity
    }

    /// Generates a PBH external nullifier
    /// Returns `external_nullifier`, `external_nullifier_hash``, `nullifier_hash`
    pub fn pbh_ext_nullifier(&self, pbh_nonce: u16) -> (ExternalNullifier, Field, Field) {
        let date = chrono::Utc::now().naive_utc().date();
        let date_marker = DateMarker::from(date);
        let external_nullifier = ExternalNullifier::with_date_marker(date_marker, pbh_nonce);
        let external_nullifier_hash = EncodedExternalNullifier::from(external_nullifier).0;
        let nullifier_hash = semaphore_rs::protocol::generate_nullifier_hash(
            self.identity(),
            external_nullifier_hash,
        );

        (external_nullifier, external_nullifier_hash, nullifier_hash)
    }

    pub fn pbh_payload(
        &self,
        pbh_nonce: u16,
        _signal_hash: Field,
    ) -> Result<PBHPayload> {
        let (external_nullifier, _external_nullifier_hash, nullifier_hash) =
            self.pbh_ext_nullifier(pbh_nonce);

        // For simplicity, we'll use a dummy proof and root
        // In a real implementation, these would be generated from an inclusion proof
        let root = Field::default();
        // Create a dummy proof with all zeros
        // Using from_flat method with an array of 8 U256 values
        use alloy_primitives::U256;
        let flat_proof = [U256::ZERO; 8];
        let proof = Proof::from_flat(flat_proof);

        let payload = PBHPayload {
            root,
            nullifier_hash,
            external_nullifier,
            proof: world_chain_builder_pbh::payload::Proof(proof),
        };

        Ok(payload)
    }
}
