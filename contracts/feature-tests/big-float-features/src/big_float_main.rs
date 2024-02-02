#![no_std]

dharitri_wasm::imports!();

pub mod big_float_methods;
pub mod big_float_operators;

#[dharitri_wasm::contract]
pub trait BigFloatFeatures:
    big_float_methods::BigFloatMethods + big_float_operators::BigFloatOperators
{
    #[init]
    fn init(&self) {}
}
