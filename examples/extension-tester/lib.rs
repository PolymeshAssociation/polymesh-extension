#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use ink_lang as ink;
use alloc::vec::Vec;

use polymesh_extension::*;

#[ink::contract(env = polymesh_extension::PolymeshEnvironment)]
mod runtime_tester {
    use alloc::vec;

    use crate::*;

    /// A simple ERC-20 contract.
    #[ink(storage)]
    pub struct RuntimeTester {}

    /// The contract error types.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Caller needs to pay the contract for the protocol fee.
        /// (Amount needed)
        InsufficientTransferValue(Balance),
        /// Polymesh runtime error.
        PolymeshError(PolymeshRuntimeErr),
    }

    // hard-code protocol fees.
    pub const POLYX: Balance = 1_000_000u128;
    pub const CREATE_ASSET_FEE: Balance = (500 * POLYX) + (2_500 * POLYX);

    /// The contract result type.
    pub type Result<T> = core::result::Result<T, Error>;

    impl RuntimeTester {
        /// Creates a new contract.
        #[ink(constructor)]
        pub fn new() -> Self { Self {} }

        #[ink(message)]
        pub fn register_ticker(&mut self, ticker: Ticker) -> Result<()> {
            Self::env().extension().register_ticker(ticker)
              .map_err(|err| Error::PolymeshError(err))
        }

        #[ink(message)]
        pub fn accept_ticker_transfer(&mut self, auth_id: u64) -> Result<()> {
            Self::env().extension().accept_ticker_transfer(auth_id)
              .map_err(|err| Error::PolymeshError(err))
        }

        #[ink(message)]
        pub fn accept_asset_ownership_transfer(&mut self, auth_id: u64) -> Result<()> {
            Self::env().extension().accept_asset_ownership_transfer(auth_id)
              .map_err(|err| Error::PolymeshError(err))
        }

        #[ink(message)]
        pub fn create_asset(&mut self, name: AssetName, ticker: Ticker, asset_type: AssetType) -> Result<()> {
            Self::env().extension().create_asset(name, ticker, true, asset_type, vec![], None, true)
              .map_err(|err| Error::PolymeshError(err))
        }

        #[ink(message, payable)]
        pub fn payable_create_asset(&mut self, name: AssetName, ticker: Ticker, asset_type: AssetType) -> Result<()> {
            let transferred = Self::env().transferred_value();
            if transferred < CREATE_ASSET_FEE {
              return Err(Error::InsufficientTransferValue(CREATE_ASSET_FEE));
            }
            Self::env().extension().create_asset(name, ticker, true, asset_type, vec![], None, true)
              .map_err(|err| Error::PolymeshError(err))
        }

        #[ink(message)]
        pub fn register_custom_asset_type(&mut self, ty: Vec<u8>) -> Result<()> {
            Self::env().extension().register_custom_asset_type(ty)
              .map_err(|err| Error::PolymeshError(err))
        }
    }
}
