use std::{
    vec::Vec,
    marker::PhantomData,
};
use frame_common::{
    crypto::{UserAddress, AccessRight, Ciphertext},
    traits::*,
    state_types::{UpdatedState, StateType},
};
use frame_runtime::traits::*;
use codec::{Encode, Decode};
use crate::error::Result;

#[derive(Debug, Clone, Encode, Decode)]
pub struct Instructions<R: RuntimeExecutor<CTX>, CTX: ContextOps> {
    my_addr: UserAddress,
    call_kind: R::C,
    phantom: PhantomData<CTX>,
}

impl<R: RuntimeExecutor<CTX, S=StateType>, CTX: ContextOps> Instructions<R, CTX> {
    pub fn new(call_id: u32, params: &mut [u8], access_right: &AccessRight) -> Result<Self> {
        let my_addr = UserAddress::from_access_right(&access_right)?;
        let call_kind = R::C::new(call_id, params)?;

        Ok(Instructions {
            my_addr,
            call_kind,
            phantom: PhantomData,
        })
    }

    pub fn encrypt<GK: GroupKeyOps>(&self, key: &GK, max_mem_size: usize) -> Result<Ciphertext> {
        // Add padding to fix the ciphertext size of all state types.
        // The padding works for fixing the ciphertext size so that
        // other people cannot distinguish what state is encrypted based on the size.
        fn append_padding(buf: &mut Vec<u8>, max_mem_size: usize) {
            let padding_size = max_mem_size - buf.len();
            let mut padding = vec![0u8; padding_size];
            buf.extend_from_slice(&mut padding);
        }

        let mut buf = self.encode();
        append_padding(&mut buf, max_mem_size);
        key.encrypt(buf).map_err(Into::into)
    }

    /// Only if the TEE belongs to the group, you can receive ciphertext and decrypt it,
    /// otherwise do nothing.
    pub fn state_transition<GK: GroupKeyOps>(
        ctx: CTX,
        ciphertext: &Ciphertext,
        group_key: &mut GK,
    ) -> Result<Option<impl Iterator<Item=UpdatedState<StateType>> + Clone>> {
        if let Some(instructions) = Instructions::<R, CTX>::decrypt(ciphertext, group_key)? {
            let state_iter = instructions
                .stf_call(ctx)?
                .into_iter();

            return Ok(Some(state_iter))
        }

        Ok(None)
    }

    fn decrypt<GK: GroupKeyOps>(ciphertext: &Ciphertext, key: &mut GK) -> Result<Option<Self>> {
        match key.decrypt(ciphertext)? {
            Some(plaintext) => {
                Instructions::decode(&mut &plaintext[..])
                    .map(|p| Some(p))
                    .map_err(Into::into)
            }
            None => Ok(None)
        }
    }

    fn stf_call(self, ctx: CTX) -> Result<Vec<UpdatedState<StateType>>> {
        let res = R::new(ctx).execute(
            self.call_kind,
            self.my_addr,
        )?;

        Ok(res)
    }
}
