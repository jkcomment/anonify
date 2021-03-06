use std::{
    vec::Vec,
    ptr,
};
use frame_common::{
    traits::{EcallInput, EcallOutput},
};
use config::constants::*;
use anonify_enclave::{
    context::EnclaveContext,
    workflow::*,
};
use erc20_state_transition::{MAX_MEM_SIZE, Runtime};
use crate::ENCLAVE_CONTEXT;
use frame_enclave::{register_ecall, EnclaveEngine};
use anyhow::anyhow;
use codec::{Encode, Decode};

register_ecall!(
    &*ENCLAVE_CONTEXT,
    MAX_MEM_SIZE,
    Runtime<EnclaveContext>,
    EnclaveContext,
    (ENCRYPT_INSTRUCTION_CMD, Instruction),
    // Insert a ciphertext in event logs from blockchain nodes into enclave's memory database.
    (INSERT_CIPHERTEXT_CMD, InsertCiphertext),
    // Insert handshake received from blockchain nodes into enclave.
    (INSERT_HANDSHAKE_CMD, InsertHandshake),
    // Get current state of the user represented the given public key from enclave memory database.
    (GET_STATE_CMD, GetState),
    (CALL_JOIN_GROUP_CMD, CallJoinGroup),
    (CALL_HANDSHAKE_CMD, CallHandshake),
    (REGISTER_NOTIFICATION_CMD, RegisterNotification),
);
