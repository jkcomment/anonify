use crate::group_state::GroupState;
use crate::application::AppKeyChain;
use crate::handshake::{Handshake, PathSecretRequest, PathSecretKVS};
use crate::crypto::{
    CryptoRng,
    secrets::PathSecret,
};
use rand_core::SeedableRng;

pub fn init_path_secret_kvs(kvs: &mut PathSecretKVS, until_roster_idx: usize, until_epoch: usize) {
    let mut csprng = rand::rngs::StdRng::seed_from_u64(1);
    for r_i in 0..until_roster_idx {
        for e_i in 0..until_epoch {
            kvs.insert_random_path_secret(r_i as u32, e_i as u32, &mut csprng);
        }
    }
}

pub fn change_group_state_idx(
    group_state: &GroupState,
    new_idx: u32,
) -> GroupState {
    assert!(new_idx as usize <= group_state.roster_len().unwrap());

    let mut new_group_state = group_state.clone();
    new_group_state.my_roster_idx = new_idx;

    new_group_state
}

pub fn do_handshake_three_party<R: CryptoRng>(
    my_group: &mut GroupState,
    others_group1: &mut GroupState,
    others_group2: &mut GroupState,
    req: &PathSecretRequest,
    csprng: &mut R,
) -> (AppKeyChain, AppKeyChain, AppKeyChain) {
    let max_roster_idx = 2;
    let new_path_secret = PathSecret::new_from_random(csprng);
    let handshake = my_group.create_handshake(req).unwrap();

    let my_keychain = my_group.process_handshake(&handshake, req, max_roster_idx).unwrap();
    let others_keychain1 = others_group1.process_handshake(&handshake, req, max_roster_idx).unwrap();
    let others_keychain2 = others_group2.process_handshake(&handshake, req, max_roster_idx).unwrap();

    (my_keychain, others_keychain1, others_keychain2)
}

pub fn encrypt_decrypt_helper(
        msg: &[u8],
        group1: &GroupState,
        app_key_chain1: &mut AppKeyChain,
        group2: &GroupState,
        app_key_chain2: &mut AppKeyChain,
        group3: &GroupState,
        app_key_chain3: &mut AppKeyChain,
    ) {
    let app_msg = app_key_chain1.encrypt_msg(msg.to_vec(), group1).unwrap();

    match app_key_chain1.decrypt_msg(&app_msg, group1).unwrap() {
        Some(plaintext1) => {
            match app_key_chain2.decrypt_msg(&app_msg, group2).unwrap() {
                Some(plaintext2) => {
                    match app_key_chain3.decrypt_msg(&app_msg, group3).unwrap() {
                        Some(plaintext3) => {
                            assert_eq!(plaintext1, plaintext2);
                            assert_eq!(plaintext2, plaintext3);
                            assert_eq!(plaintext3.as_slice(), msg);
                        }
                        None => { }
                    }
                }
                None => { }
            }
        },
        None => { }
    };
}

