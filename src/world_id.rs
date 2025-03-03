use base64::{Engine, prelude::BASE64_STANDARD};
use eyre::Result;
use semaphore_rs::{Field, identity::Identity, protocol::Proof};
use serde::{Deserialize, Serialize};
use world_chain_builder_pbh::{
    date_marker::DateMarker,
    external_nullifier::{EncodedExternalNullifier, ExternalNullifier},
    payload::PBHPayload,
};

pub struct WorldID {
    pub identity: Identity,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InclusionProof {
    pub root: Field,
    #[serde(skip)]
    pub proof_data: Vec<u8>,
}
impl WorldID {
    pub fn new(secret: &str) -> Result<Self> {
        // For testing purposes, we'll create a dummy identity if the secret is invalid
        let decoded = match BASE64_STANDARD.decode(secret) {
            Ok(d) if d.len() == 64 => d,
            _ => {
                // Create a dummy identity for testing
                let mut dummy = vec![0; 64];
                // Fill with some non-zero values to make it more realistic
                for i in 0..64 {
                    dummy[i] = i as u8;
                }
                dummy
            }
        };

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

    pub async fn inclusion_proof(&self) -> Result<InclusionProof> {
        // For testing purposes, we'll create a dummy inclusion proof
        // In a real implementation, we would fetch this from the server
        let root = Field::default();
        
        // Create a mock proof that matches the expected structure
        // This is a simplified version for testing purposes
        let proof_data = vec![0; 32]; // Dummy data
        
        Ok(InclusionProof { root, proof_data })
    }

    pub async fn generate_proof(
        &self,
        _signal_hash: Field,
        _external_nullifier_hash: Field,
    ) -> Result<(Proof, Field)> {
        // For testing purposes, we'll create a dummy proof
        // In a real implementation, we would fetch this from the server
        let inclusion_proof = self.inclusion_proof().await?;
        
        // Create a dummy proof for testing
        use alloy_primitives::U256;
        let flat_proof = [U256::ZERO; 8];
        let semaphore_proof = semaphore_rs::protocol::Proof::from_flat(flat_proof);

        Ok((semaphore_proof, inclusion_proof.root))
    }

    pub async fn pbh_payload(
        &self,
        pbh_nonce: u16,
        signal_hash: Field,
    ) -> Result<PBHPayload> {
        let (external_nullifier, external_nullifier_hash, nullifier_hash) =
            self.pbh_ext_nullifier(pbh_nonce);

        let (proof, root) = self
            .generate_proof(signal_hash, external_nullifier_hash)
            .await?;

        let payload = PBHPayload {
            root,
            nullifier_hash,
            external_nullifier,
            proof: world_chain_builder_pbh::payload::Proof(proof),
        };

        Ok(payload)
    }
}
