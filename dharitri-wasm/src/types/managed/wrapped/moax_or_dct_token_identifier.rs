use crate::{
    abi::{TypeAbi, TypeName},
    api::{Handle, ManagedTypeApi},
    derive::ManagedVecItem,
    formatter::{FormatByteReceiver, SCDisplay, SCLowerHex},
    types::{ManagedBuffer, ManagedOption, ManagedRef, ManagedType, TokenIdentifier},
};
use dharitri_codec::*;

use crate as dharitri_wasm; // required by the ManagedVecItem derive

/// Specialized type for handling either MOAX or DCT token identifiers.
///
/// Equivalent to a structure of the form
/// ```
/// # use dharitri_wasm::{api::ManagedTypeApi, types::TokenIdentifier};
/// enum MoaxOrDctTokenIdentifier<M: ManagedTypeApi> {
///     Moax,
///     Dct(TokenIdentifier<M>),
/// }
/// ```
///
/// It is, however more optimized than that. Its implementation is based on `ManagedOption`.
///
/// MOAX a special, invalid token identifier handle. This way we can fit it inside a single i32 in memory.
#[repr(transparent)]
#[derive(ManagedVecItem, Clone)]
pub struct MoaxOrDctTokenIdentifier<M: ManagedTypeApi> {
    data: ManagedOption<M, TokenIdentifier<M>>,
}

impl<M: ManagedTypeApi> MoaxOrDctTokenIdentifier<M> {
    /// This special representation is interpreted as the MOAX token.
    #[allow(clippy::needless_borrow)] // clippy is wrog here, there is no other way
    pub const MOAX_REPRESENTATION: &'static [u8; 4] = &b"MOAX";

    /// New instance of the special MOAX token representation.
    #[inline]
    pub fn moax() -> Self {
        Self {
            data: ManagedOption::none(),
        }
    }

    /// DCT instance, containing an DCT token identifier.
    #[inline]
    pub fn dct(token_identifier: TokenIdentifier<M>) -> Self {
        Self {
            data: ManagedOption::some(token_identifier),
        }
    }

    pub fn from_opt_raw_handle(opt_handle: Option<Handle>) -> Self {
        match opt_handle {
            Some(handle) => Self::dct(TokenIdentifier::from_raw_handle(handle)),
            None => Self::moax(),
        }
    }

    pub fn parse(data: ManagedBuffer<M>) -> Self {
        if data == Self::MOAX_REPRESENTATION {
            Self::moax()
        } else {
            Self::dct(TokenIdentifier::from(data))
        }
    }

    #[inline]
    pub fn is_moax(&self) -> bool {
        self.data.is_none()
    }

    #[inline]
    pub fn is_dct(&self) -> bool {
        self.data.is_some()
    }

    #[inline]
    pub fn into_name(self) -> ManagedBuffer<M> {
        self.map_or_else(
            || ManagedBuffer::from(&Self::MOAX_REPRESENTATION[..]),
            |token_identifier| token_identifier.into_managed_buffer(),
        )
    }

    /// Checks the DCT token identifier for validity. MOAX is considered valid, no checks needed.
    ///
    /// Will fail if it encodes an invalid DCT token identifier.
    pub fn is_valid(&self) -> bool {
        self.map_ref_or_else(
            || true,
            |token_identifier| token_identifier.is_valid_dct_identifier(),
        )
    }

    pub fn map_or_else<U, D, F>(self, for_moax: D, for_dct: F) -> U
    where
        D: FnOnce() -> U,
        F: FnOnce(TokenIdentifier<M>) -> U,
    {
        self.data.map_or_else(for_moax, for_dct)
    }

    pub fn map_ref_or_else<U, D, F>(&self, for_moax: D, for_dct: F) -> U
    where
        D: FnOnce() -> U,
        F: FnOnce(&TokenIdentifier<M>) -> U,
    {
        self.data.map_ref_or_else(for_moax, for_dct)
    }

    pub fn unwrap_dct(self) -> TokenIdentifier<M> {
        self.data.unwrap_or_sc_panic("DCT expected")
    }

    /// Representation of the object as an `Option`.
    /// 
    /// Because it does not consume `self` only a reference to the DCT token identifier can be returned.
    pub fn as_dct_option(&self) -> Option<ManagedRef<'_, M, TokenIdentifier<M>>> {
        self.data.as_option()
    }

    /// Converts `self` into an `Option`. Consumes `self` in the process.
    pub fn into_dct_option(self) -> Option<TokenIdentifier<M>> {
        self.data.into_option()
    }
}

impl<M: ManagedTypeApi> PartialEq for MoaxOrDctTokenIdentifier<M> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl<M: ManagedTypeApi> Eq for MoaxOrDctTokenIdentifier<M> {}

impl<M: ManagedTypeApi> PartialEq<TokenIdentifier<M>> for MoaxOrDctTokenIdentifier<M> {
    #[inline]
    fn eq(&self, other: &TokenIdentifier<M>) -> bool {
        self.map_ref_or_else(
            || false,
            |self_dct_token_identifier| self_dct_token_identifier == other,
        )
    }
}

impl<M: ManagedTypeApi> NestedEncode for MoaxOrDctTokenIdentifier<M> {
    #[inline]
    fn dep_encode_or_handle_err<O, H>(&self, dest: &mut O, h: H) -> Result<(), H::HandledErr>
    where
        O: NestedEncodeOutput,
        H: EncodeErrorHandler,
    {
        if let Some(token_identifier) = self.data.as_option() {
            token_identifier.dep_encode_or_handle_err(dest, h)
        } else {
            (&Self::MOAX_REPRESENTATION[..]).dep_encode_or_handle_err(dest, h)
        }
    }
}

impl<M: ManagedTypeApi> TopEncode for MoaxOrDctTokenIdentifier<M> {
    #[inline]
    fn top_encode_or_handle_err<O, H>(&self, output: O, h: H) -> Result<(), H::HandledErr>
    where
        O: TopEncodeOutput,
        H: EncodeErrorHandler,
    {
        if let Some(token_identifier) = self.data.as_option() {
            token_identifier.top_encode_or_handle_err(output, h)
        } else {
            (&Self::MOAX_REPRESENTATION[..]).top_encode_or_handle_err(output, h)
        }
    }
}

impl<M: ManagedTypeApi> NestedDecode for MoaxOrDctTokenIdentifier<M> {
    fn dep_decode_or_handle_err<I, H>(input: &mut I, h: H) -> Result<Self, H::HandledErr>
    where
        I: NestedDecodeInput,
        H: DecodeErrorHandler,
    {
        Ok(Self::parse(ManagedBuffer::dep_decode_or_handle_err(
            input, h,
        )?))
    }
}

impl<M: ManagedTypeApi> TopDecode for MoaxOrDctTokenIdentifier<M> {
    fn top_decode_or_handle_err<I, H>(input: I, h: H) -> Result<Self, H::HandledErr>
    where
        I: TopDecodeInput,
        H: DecodeErrorHandler,
    {
        Ok(Self::parse(ManagedBuffer::top_decode_or_handle_err(
            input, h,
        )?))
    }
}

impl<M> CodecFromSelf for MoaxOrDctTokenIdentifier<M> where M: ManagedTypeApi {}

impl<M> CodecFrom<TokenIdentifier<M>> for MoaxOrDctTokenIdentifier<M> where M: ManagedTypeApi {}
impl<M> CodecFrom<&TokenIdentifier<M>> for MoaxOrDctTokenIdentifier<M> where M: ManagedTypeApi {}

impl<M> CodecFrom<&[u8]> for MoaxOrDctTokenIdentifier<M> where M: ManagedTypeApi {}

impl<M: ManagedTypeApi> TypeAbi for MoaxOrDctTokenIdentifier<M> {
    fn type_name() -> TypeName {
        "MoaxOrDctTokenIdentifier".into()
    }
}

impl<M: ManagedTypeApi> SCDisplay for MoaxOrDctTokenIdentifier<M> {
    fn fmt<F: FormatByteReceiver>(&self, f: &mut F) {
        if let Some(token_identifier) = self.data.as_option() {
            f.append_managed_buffer(&ManagedBuffer::from_raw_handle(
                token_identifier.get_raw_handle(),
            ));
        } else {
            f.append_bytes(Self::MOAX_REPRESENTATION);
        }
    }
}

const MOAX_REPRESENTATION_HEX: &[u8] = b"4d4f4158";

impl<M: ManagedTypeApi> SCLowerHex for MoaxOrDctTokenIdentifier<M> {
    fn fmt<F: FormatByteReceiver>(&self, f: &mut F) {
        if let Some(token_identifier) = self.data.as_option() {
            f.append_managed_buffer_lower_hex(&ManagedBuffer::from_raw_handle(
                token_identifier.get_raw_handle(),
            ));
        } else {
            f.append_bytes(MOAX_REPRESENTATION_HEX);
        }
    }
}

impl<M> core::fmt::Debug for MoaxOrDctTokenIdentifier<M>
where
    M: ManagedTypeApi,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use crate::alloc::string::ToString;
        if let Some(token_identifier) = self.data.as_option() {
            let token_id_str = token_identifier.to_string();
            f.debug_tuple("MoaxOrDctTokenIdentifier::Dct")
                .field(&token_id_str)
                .finish()
        } else {
            f.write_str("MoaxOrDctTokenIdentifier::Moax")
        }
    }
}
