use std::vec::Vec;
use std::convert::TryFrom;
use crate::group_state::GroupState;
use crate::crypto::{
    secrets::{AppSecret, AppMemberSecret, HmacKey},
    hkdf, SHA256_OUTPUT_LEN,
};
use anyhow::{Result, anyhow};
use codec::{Encode};

#[derive(Clone, Debug)]
pub struct AppMsg {
    generation: u32,
    encrypted_msg: Vec<u8>,
}

pub struct AppKeyChain {
    member_secrets_and_gens: Vec<(AppMemberSecret, u32)>,
}

impl AppKeyChain {
    pub fn from_app_secret(group_state: &GroupState, app_secret: AppSecret) -> Self {
        let roster_len = u32::try_from(group_state.roster_len().expect("Invalid roster length"))
                .expect("roster length exceeds u32::MAX");
        let prk = HmacKey::from(app_secret);

        let member_secrets_and_gens = (0..roster_len).map(|roster_idx: u32| {
            let mut buf = vec![0u8; SHA256_OUTPUT_LEN];
            let encoded_roster_idx = roster_idx.encode();
            hkdf::expand_label(
                &prk,
                b"app sender",
                &encoded_roster_idx,
                buf.as_mut_slice(),
            )
            .expect("Failed hkdf expand.");
            let app_member_secret = AppMemberSecret::from(buf);

            (app_member_secret, 0)
        })
        .collect();

        AppKeyChain { member_secrets_and_gens }
    }

    pub fn ratchet(&mut self, roster_idx: usize) -> Result<()> {
        let (member_secret, gen) = self.member_secrets_and_gens
            .get_mut(roster_idx)
            .ok_or(anyhow!("Roster index is out of range of application key chain"))?;
        let current_secret = member_secret.clone();

        let roster_idx = u32::try_from(roster_idx)?;
        hkdf::expand_label(
            &current_secret.into(),
            b"app sender",
            &roster_idx.encode(),
            member_secret.as_mut_bytes(),
        );

        *gen = gen.checked_add(1).ok_or(anyhow!("geenration is over u32::MAX"))?;

        Ok(())
    }

    pub fn encrypt_msg(
        &mut self,
        plaintext: Vec<u8>,
        group_state: &GroupState
    ) -> Result<AppMsg> {
        let my_roster_index = group_state.my_roster_index();
        unimplemented!();
    }

    pub fn decrypt_msg(
        &mut self,
        mut app_msg: AppMsg,
        group_state: &GroupState,
    ) -> Result<Vec<u8>> {
        unimplemented!();
    }
}
