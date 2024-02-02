use super::{ErrorApiImpl, Handle, ManagedTypeApiImpl};
use crate::types::DctTokenType;

pub trait CallValueApi {
    type CallValueApiImpl: CallValueApiImpl;

    fn call_value_api_impl() -> Self::CallValueApiImpl;
}

pub trait CallValueApiImpl: ErrorApiImpl + ManagedTypeApiImpl + Sized {
    fn check_not_payable(&self);

    /// Retrieves the MOAX call value from the VM.
    /// Will return 0 in case of an DCT transfer (cannot have both MOAX and DCT transfer simultaneously).
    fn load_moax_value(&self, dest_handle: Handle);

    /// Loads all DCT call values into a managed vec. Overwrites destination.
    fn load_all_dct_transfers(&self, dest_handle: Handle) {
        load_all_dct_transfers_from_unmanaged(self, dest_handle);
    }

    fn dct_num_transfers(&self) -> usize;

    /// Retrieves the DCT call value from the VM.
    /// Will return 0 in case of an MOAX transfer (cannot have both MOAX and DCT transfer simultaneously).
    fn load_single_dct_value(&self, dest_handle: Handle);

    /// Returns the call value token identifier of the current call.
    /// The identifier is wrapped in a TokenIdentifier object, to hide underlying logic.
    fn token(&self) -> Option<Handle>;

    /// Returns the nonce of the received DCT token.
    /// Will return 0 in case of MOAX or fungible DCT transfer.
    fn dct_token_nonce(&self) -> u64;

    /// Returns the DCT token type.
    /// Will return "Fungible" for MOAX.
    fn dct_token_type(&self) -> DctTokenType;

    fn dct_value_by_index(&self, index: usize) -> Handle;

    fn token_by_index(&self, index: usize) -> Handle;

    fn dct_token_nonce_by_index(&self, index: usize) -> u64;

    fn dct_token_type_by_index(&self, index: usize) -> DctTokenType;
}

pub fn load_all_dct_transfers_from_unmanaged<A>(api: &A, dest_handle: Handle)
where
    A: CallValueApiImpl,
{
    let num_transfers = api.dct_num_transfers();
    api.mb_overwrite(dest_handle, &[]);

    for i in 0..num_transfers {
        let token_identifier_handle = api.token_by_index(i);
        let token_nonce = api.dct_token_nonce_by_index(i);
        let amount_handle = api.dct_value_by_index(i);

        api.mb_append_bytes(dest_handle, &token_identifier_handle.to_be_bytes()[..]);
        api.mb_append_bytes(dest_handle, &token_nonce.to_be_bytes()[..]);
        api.mb_append_bytes(dest_handle, &amount_handle.to_be_bytes()[..]);
    }
}
