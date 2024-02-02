use core::marker::PhantomData;

use crate::{
    api::{
        const_handles, CallValueApi, CallValueApiImpl, ErrorApi, ErrorApiImpl, ManagedTypeApi,
        StaticVarApiImpl,
    },
    err_msg,
    types::{
        BigUint, MoaxOrDctTokenIdentifier, MoaxOrDctTokenPayment, DctTokenPayment, ManagedType,
        ManagedVec, TokenIdentifier,
    },
};

#[derive(Default)]
pub struct CallValueWrapper<A>
where
    A: CallValueApi + ErrorApi + ManagedTypeApi,
{
    _phantom: PhantomData<A>,
}

impl<A> CallValueWrapper<A>
where
    A: CallValueApi + ErrorApi + ManagedTypeApi,
{
    pub fn new() -> Self {
        CallValueWrapper {
            _phantom: PhantomData,
        }
    }

    /// Retrieves the MOAX call value from the VM.
    /// Will return 0 in case of an DCT transfer (cannot have both MOAX and DCT transfer simultaneously).
    pub fn moax_value(&self) -> BigUint<A> {
        let mut call_value_handle = A::static_var_api_impl().get_call_value_moax_handle();
        if call_value_handle == const_handles::UNINITIALIZED_HANDLE {
            call_value_handle = const_handles::CALL_VALUE_MOAX;
            A::static_var_api_impl().set_call_value_moax_handle(call_value_handle);
            A::call_value_api_impl().load_moax_value(call_value_handle);
        }
        BigUint::from_raw_handle(call_value_handle) // unsafe, TODO: replace with ManagedRef<...>
    }

    /// Returns all DCT transfers that accompany this SC call.
    /// Will return 0 results if nothing was transfered, or just MOAX.
    /// Fully managed underlying types, very efficient.
    pub fn all_dct_transfers(&self) -> ManagedVec<A, DctTokenPayment<A>> {
        let mut call_value_handle = A::static_var_api_impl().get_call_value_multi_dct_handle();
        if call_value_handle == const_handles::UNINITIALIZED_HANDLE {
            call_value_handle = const_handles::CALL_VALUE_MULTI_DCT;
            A::static_var_api_impl().set_call_value_multi_dct_handle(call_value_handle);
            A::call_value_api_impl().load_all_dct_transfers(call_value_handle);
        }
        ManagedVec::from_raw_handle(call_value_handle) // unsafe, TODO: replace with ManagedRef<...>
    }

    /// Verify and casts the received multi DCT transfer in to an array.
    ///
    /// Can be used to extract all payments in one line like this:
    ///
    /// `let [payment_a, payment_b, payment_c] = self.call_value().multi_dct();`.
    pub fn multi_dct<const N: usize>(&self) -> [DctTokenPayment<A>; N] {
        self.all_dct_transfers()
            .to_array_of_refs::<N>()
            .unwrap_or_else(|| {
                A::error_api_impl().signal_error(err_msg::INCORRECT_NUM_DCT_TRANSFERS.as_bytes())
            })
    }

    /// Expects precisely one DCT token transfer, fungible or not.
    ///
    /// Will return the received DCT payment.
    ///
    /// The amount cannot be 0, since that would not qualify as an DCT transfer.
    pub fn single_dct(&self) -> DctTokenPayment<A> {
        let [payments] = self.multi_dct();
        payments
    }

    /// Expects precisely one fungible DCT token transfer.
    ///
    /// Returns the token ID and the amount for fungible DCT transfers.
    ///
    /// The amount cannot be 0, since that would not qualify as an DCT transfer.
    pub fn single_fungible_dct(&self) -> (TokenIdentifier<A>, BigUint<A>) {
        let payment = self.single_dct();
        if payment.token_nonce != 0 {
            A::error_api_impl().signal_error(err_msg::FUNGIBLE_TOKEN_EXPECTED_ERR_MSG.as_bytes());
        }
        (payment.token_identifier, payment.amount)
    }

    /// Retrieves the DCT call value from the VM.
    /// Will return 0 in case of an MOAX transfer (cannot have both MOAX and DCT transfer simultaneously).
    pub fn dct_value(&self) -> BigUint<A> {
        A::call_value_api_impl().load_single_dct_value(const_handles::CALL_VALUE_SINGLE_DCT);
        BigUint::from_raw_handle(const_handles::CALL_VALUE_SINGLE_DCT)
    }

    /// Accepts and returns either an MOAX payment, or a single DCT token.
    ///
    /// Will halt execution if more than one DCT transfer was received.
    ///
    /// In case no transfer of value happen, it will return a payment of 0 MOAX.
    pub fn moax_or_single_dct(&self) -> MoaxOrDctTokenPayment<A> {
        let dct_transfers = self.all_dct_transfers();
        match dct_transfers.len() {
            0 => MoaxOrDctTokenPayment {
                token_identifier: MoaxOrDctTokenIdentifier::moax(),
                token_nonce: 0,
                amount: self.moax_value(),
            },
            1 => dct_transfers.get(0).into(),
            _ => A::error_api_impl().signal_error(err_msg::INCORRECT_NUM_DCT_TRANSFERS.as_bytes()),
        }
    }

    /// Accepts and returns either an MOAX payment, or a single fungible DCT token.
    ///
    /// Will halt execution if more than one DCT transfer was received, or if the received DCT is non- or semi-fungible.
    ///
    /// Works similar to `moax_or_single_dct`,
    /// but checks the nonce to be 0 and returns a tuple of just token identifier and amount, for convenience.
    ///
    /// In case no transfer of value happen, it will return a payment of 0 MOAX.
    pub fn moax_or_single_fungible_dct(&self) -> (MoaxOrDctTokenIdentifier<A>, BigUint<A>) {
        let payment = self.moax_or_single_dct();
        if payment.token_nonce != 0 {
            A::error_api_impl().signal_error(err_msg::FUNGIBLE_TOKEN_EXPECTED_ERR_MSG.as_bytes());
        }

        (payment.token_identifier, payment.amount)
    }
}