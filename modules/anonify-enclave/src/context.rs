use std::{
    sync::{SgxRwLock, SgxRwLockReadGuard, SgxRwLockWriteGuard, Arc},
    env,
};
use sgx_types::*;
use std::prelude::v1::*;
use frame_common::{
    crypto::UserAddress,
    traits::*,
    state_types::{MemId, UpdatedState, StateType},
};
use frame_runtime::traits::*;
use frame_treekem::{
    handshake::{PathSecretRequest, PathSecretKVS},
    init_path_secret_kvs,
};
use frame_enclave::ocalls::{sgx_init_quote, get_quote};
use crate::{
    notify::Notifier,
    crypto::EnclaveIdentityKey,
    config::{UNTIL_ROSTER_IDX, UNTIL_EPOCH},
    error::Result,
    kvs::EnclaveDB,
    group_key::GroupKey,
};

impl StateOps for EnclaveContext {
    type S = StateType;

    fn get_state<U>(&self, key: U, mem_id: MemId) -> Self::S
    where
        U: Into<UserAddress>,
    {
        self.db
            .get(key.into(), mem_id)
    }

    /// Returns a updated state of registerd address in notification.
    // TODO: Enables to return multiple updated states.
    fn update_state(
        &self,
        mut state_iter: impl Iterator<Item=UpdatedState<Self::S>> + Clone
    ) -> Option<UpdatedState<Self::S>> {
        state_iter.clone().for_each(|s| self.db.insert_by_updated_state(s));
        state_iter.find(|s| self.is_notified(&s.address))
    }
}

impl GroupKeyGetter for EnclaveContext {
    type GK = GroupKey;

    fn read_group_key(&self) -> SgxRwLockReadGuard<Self::GK> {
        self.group_key.read().unwrap()
    }

    fn write_group_key(&self) -> SgxRwLockWriteGuard<Self::GK> {
        self.group_key.write().unwrap()
    }
}

impl NotificationOps for EnclaveContext {
    fn set_notification(&self, address: UserAddress) -> bool {
        self.notifier.register(address)
    }

    fn is_notified(&self, address: &UserAddress) -> bool {
        self.notifier.contains(&address)
    }
}

impl Signer for EnclaveContext {
    /// Generate a signature using enclave's identity key.
    /// This signature is used to verify enclave's program dependencies and
    /// should be verified in the public available place such as smart contract on blockchain.
    fn sign(&self, msg: &[u8]) -> anyhow::Result<secp256k1::Signature> {
        self.identity_key.sign(msg).map_err(Into::into)
    }
}

impl QuoteGetter for EnclaveContext {
    fn quote(&self) -> anyhow::Result<String> {
        let target_info = self.init_quote()?;
        let report = self.report(&target_info)?;
        self.encoded_quote(report).map_err(Into::into)
    }
}

/// spid: Service provider ID for the ISV.
#[derive(Clone)]
pub struct EnclaveContext {
    spid: sgx_spid_t,
    identity_key: EnclaveIdentityKey,
    db: EnclaveDB,
    notifier: Notifier,
    pub group_key: Arc<SgxRwLock<GroupKey>>,
}

// TODO: Consider SGX_ERROR_BUSY.
impl EnclaveContext {
    pub fn new(spid: &str) -> Result<Self> {
        let spid_vec = hex::decode(spid)?;
        let mut id = [0; 16];
        id.copy_from_slice(&spid_vec);
        let spid: sgx_spid_t = sgx_spid_t { id };

        let identity_key = EnclaveIdentityKey::new()?;
        let db = EnclaveDB::new();

        // temporary path secrets are generated in local.
        let mut kvs = PathSecretKVS::new();
        init_path_secret_kvs(&mut kvs, UNTIL_ROSTER_IDX, UNTIL_EPOCH);
        let req = PathSecretRequest::Local(kvs);

        let my_roster_idx: usize = env::var("MY_ROSTER_IDX")
            .expect("MY_ROSTER_IDX is not set")
            .parse()
            .expect("Failed to parse MY_ROSTER_IDX to usize");
        let max_roster_idx: usize = env::var("MAX_ROSTER_IDX")
            .expect("MAX_ROSTER_IDX is not set")
            .parse()
            .expect("Failed to parse MAX_ROSTER_IDX to usize");

        let group_key = Arc::new(SgxRwLock::new(GroupKey::new(my_roster_idx, max_roster_idx, req)?));
        let notifier = Notifier::new();

        Ok(EnclaveContext{
            spid,
            identity_key,
            db,
            notifier,
            group_key,
        })
    }

    pub(crate) fn init_quote(&self) -> Result<sgx_target_info_t> {
        let target_info = sgx_init_quote()?;
        Ok(target_info)
    }

    /// Return Attestation report
    fn report(&self, target_info: &sgx_target_info_t) -> Result<sgx_report_t> {
        let mut report = sgx_report_t::default();
        let report_data = &self.identity_key.report_data()?;

        if let Ok(r) = sgx_tse::rsgx_create_report(&target_info, &report_data) {
            report = r;
        }

        Ok(report)
    }

    fn encoded_quote(&self, report: sgx_report_t) -> Result<String> {
        let quote = get_quote(report, &self.spid)?;

        // Use base64-encoded QUOTE structure to communicate via defined API.
        Ok(base64::encode(&quote))
    }
}
