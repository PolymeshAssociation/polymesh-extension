#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use ink_lang as ink;

use polymesh_api::{
  Api,
  ink::{
    extension::PolymeshEnvironment,
    basic_types::IdentityId,
    Error as PolymeshError,
  },
  polymesh::types::{
    pallet_settlement::{
      VenueId,
      VenueDetails,
      VenueType,
      SettlementType,
      Leg,
    },
    pallet_portfolio::{
      MovePortfolioItem
    },
    polymesh_primitives::{
      secondary_key::KeyRecord,
      ticker::Ticker,
      asset::{
        AssetName,
        AssetType,
      },
      identity_id::{
        PortfolioId,
        PortfolioKind,
        PortfolioNumber,
      },
    },
  },
};

#[ink::contract(env = PolymeshEnvironment)]
mod settlements {
    use ink_storage::{
        traits::{
            PackedLayout,
            SpreadLayout,
            SpreadAllocate,
        },
        Mapping,
    };
    use alloc::vec;

    use crate::*;

    pub const UNIT: Balance = 1_000_000u128;

    #[derive(Clone, Copy, Default, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[derive(SpreadAllocate, SpreadLayout, PackedLayout)]
    #[cfg_attr(
        feature = "std",
        derive(Debug, scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    pub struct Ticker(pub [u8; 12]);

    impl From<super::Ticker> for Ticker {
      fn from(t: super::Ticker) -> Self {
        Self(t.0)
      }
    }

    impl From<Ticker> for super::Ticker {
      fn from(t: Ticker) -> Self {
        Self(t.0)
      }
    }

    #[derive(Clone, Copy, Default, scale::Encode, scale::Decode)]
    #[derive(SpreadAllocate, SpreadLayout, PackedLayout)]
    #[cfg_attr(
        feature = "std",
        derive(Debug, scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    pub struct VenueId(u64);

    impl From<super::VenueId> for VenueId {
      fn from(v: super::VenueId) -> Self {
        Self(v.0)
      }
    }

    impl From<VenueId> for super::VenueId {
      fn from(v: VenueId) -> Self {
        Self(v.0)
      }
    }

    #[derive(Clone, Copy, Default, scale::Encode, scale::Decode)]
    #[derive(SpreadLayout, PackedLayout)]
    #[cfg_attr(
        feature = "std",
        derive(Debug, scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    pub struct PortfolioNumber(u64);

    impl From<super::PortfolioNumber> for PortfolioNumber {
      fn from(v: super::PortfolioNumber) -> Self {
        Self(v.0)
      }
    }

    impl From<PortfolioNumber> for super::PortfolioNumber {
      fn from(v: PortfolioNumber) -> Self {
        Self(v.0)
      }
    }

    #[derive(Clone, Copy, scale::Encode, scale::Decode)]
    #[derive(SpreadLayout, PackedLayout)]
    #[cfg_attr(
        feature = "std",
        derive(Debug, scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    pub enum PortfolioKind {
      Default,
      User(PortfolioNumber),
    }

    impl Default for PortfolioKind {
      fn default() -> Self {
        Self::Default
      }
    }

    impl From<super::PortfolioKind> for PortfolioKind {
      fn from(v: super::PortfolioKind) -> Self {
        match v {
          super::PortfolioKind::Default => Self::Default,
          super::PortfolioKind::User(num) => Self::User(num.into()),
        }
      }
    }

    impl From<PortfolioKind> for super::PortfolioKind {
      fn from(v: PortfolioKind) -> Self {
        match v {
          PortfolioKind::Default => super::PortfolioKind::Default,
          PortfolioKind::User(num) => super::PortfolioKind::User(num.into()),
        }
      }
    }

    #[derive(Clone, Copy, Default, scale::Encode, scale::Decode)]
    #[derive(SpreadLayout, PackedLayout)]
    #[cfg_attr(
        feature = "std",
        derive(Debug, scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    pub struct PortfolioId {
      pub did: IdentityId,
      pub kind: PortfolioKind,
    }

    impl PortfolioId {
      pub fn default(did: IdentityId) -> Self {
        Self {
          did,
          kind: PortfolioKind::Default,
        }
      }

      pub fn kind(did: IdentityId, kind: PortfolioKind) -> Self {
        Self {
          did,
          kind,
        }
      }
    }

    impl From<super::PortfolioId> for PortfolioId {
      fn from(v: super::PortfolioId) -> Self {
        Self {
          did: v.did,
          kind: v.kind.into(),
        }
      }
    }

    impl From<PortfolioId> for super::PortfolioId {
      fn from(v: PortfolioId) -> Self {
        Self {
          did: v.did,
          kind: v.kind.into(),
        }
      }
    }

    /// A contract that uses the settlements pallet.
    #[ink(storage)]
    #[derive(Default, SpreadAllocate)]
    pub struct Settlements {
      ticker1: Ticker,
      ticker2: Ticker,
      initialized: bool,
      /// Venue for settlements.
      venue: VenueId,
      /// Contract's identity.
      did: IdentityId,
      /// Custodial portfolios.
      portfolios: Mapping<IdentityId, PortfolioId>,
    }

    /// The contract error types.
    #[derive(Debug, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Caller needs to pay the contract for the protocol fee.
        /// (Amount needed)
        InsufficientTransferValue(Balance),
        /// Polymesh runtime error.
        PolymeshError(PolymeshError),
        /// Scale decode failed.
        ScaleError,
        /// Missing Identity.  MultiSig's are not supported.
        MissingIdentity,
        /// Contract hasn't been initialized.
        NotInitialized,
        /// Contract has already been initialized.
        AlreadyInitialized,
        /// Invalid portfolio authorization.
        InvalidPortfolioAuthorization,
        /// The caller has already initialized a portfolio.
        AlreadyHavePortfolio,
        /// The caller doesn't have a portfolio yet.
        NoPortfolio,
        /// Invalid ticker.
        InvalidTicker,
    }

    impl From<PolymeshError> for Error {
      fn from(err: PolymeshError) -> Self {
        Self::PolymeshError(err)
      }
    }

    /// The contract result type.
    pub type Result<T> = core::result::Result<T, Error>;

    impl Settlements {
        /// Creates a new contract.
        #[ink(constructor)]
        pub fn new(ticker1: Ticker, ticker2: Ticker) -> Self {
          ink_lang::utils::initialize_contract(|contract| {
              Self::new_init(contract, ticker1, ticker2)
          })
        }

        fn new_init(&mut self, ticker1: Ticker, ticker2: Ticker) {
          self.ticker1 = ticker1;
          self.ticker2 = ticker2;
          // The contract should always have an identity.
          self.did = self.get_did(Self::env().account_id()).unwrap();
          self.initialized = false;
        }

        fn create_asset(&mut self, ticker: Ticker) -> Result<()> {
            let api = Api::new();
            // Create asset.
            api.call().asset().create_asset(
              AssetName(b"".to_vec()),
              ticker.into(),
              true, // Divisible token.
              AssetType::EquityCommon,
              vec![],
              None,
              true // Disable Investor uniqueness requirements.
            ).submit()?;
            // Mint some tokens.
            api.call().asset().issue(ticker.into(), 1_000_000 * UNIT).submit()?;
            // Pause compliance rules to allow transfers.
            api.call().compliance_manager().pause_asset_compliance(ticker.into()).submit()?;
            Ok(())
        }

        fn get_did(&self, acc: AccountId) -> Result<IdentityId> {
            let api = Api::new();
            match api.query().identity().key_records(acc)? {
              Some(KeyRecord::PrimaryKey(did)) => Ok(did.into()),
              Some(KeyRecord::SecondaryKey(did, _)) => Ok(did.into()),
              _ => Err(Error::MissingIdentity),
            }
        }

        fn get_caller_did(&self) -> Result<IdentityId> {
          self.get_did(Self::env().caller())
        }

        fn ensure_ticker(&self, ticker: Ticker) -> Result<()> {
          if self.ticker1 != ticker && self.ticker2 != ticker {
            Err(Error::InvalidTicker)
          } else {
            Ok(())
          }
        }

        fn ensure_has_portfolio(&self, did: IdentityId) -> Result<PortfolioId> {
          self.portfolios.get(did).ok_or(Error::NoPortfolio)
        }

        fn ensure_no_portfolio(&self, did: IdentityId) -> Result<()> {
            if self.portfolios.get(did).is_some() {
              return Err(Error::AlreadyHavePortfolio);
            }
            Ok(())
        }

        fn ensure_initialized(&self) -> Result<()> {
            if !self.initialized {
              return Err(Error::NotInitialized);
            }
            Ok(())
        }

        fn init_venue(&mut self) -> Result<()> {
            if self.initialized {
              return Err(Error::AlreadyInitialized);
            }
            // Create tickers.
            self.create_asset(self.ticker1)?;
            self.create_asset(self.ticker2)?;

            let api = Api::new();
            // Get the next venue id.
            let id = api.query().settlement().venue_counter()
              .map(|v| v.into())?;
            // Create Venue.
            api.call().settlement().create_venue(
              VenueDetails(b"Contract Venue".to_vec()),
              vec![],
              VenueType::Other
            ).submit()?;
            // Save venue id.
            self.venue = id;
            self.initialized = true;
            Ok(())
        }

        #[ink(message)]
        pub fn init(&mut self) -> Result<()> {
            self.init_venue()
        }

        #[ink(message)]
        pub fn venue(&self) -> Result<VenueId> {
            self.ensure_initialized()?;
            Ok(self.venue)
        }

        #[ink(message)]
        pub fn contract_did(&self) -> Result<IdentityId> {
            self.ensure_initialized()?;
            Ok(self.did)
        }

        fn fund_caller(&self) -> Result<()> {
            // Get the caller's identity.
            let caller_did = self.get_caller_did()?;

            // Ensure the caller has a portfolio.
            let caller_portfolio = self.ensure_has_portfolio(caller_did)?;

            let api = Api::new();
            // Transfer some tokens to the caller's portfolio.
            let our_portfolio = PortfolioId::default(self.did);
            api.call().settlement().add_and_affirm_instruction(
              self.venue.into(),
              SettlementType::SettleOnAffirmation,
              None,
              None,
              vec![Leg {
                from: our_portfolio.into(),
                to: caller_portfolio.into(),
                asset: self.ticker1.into(),
                amount: 10 * UNIT,
              }, Leg {
                from: our_portfolio.into(),
                to: caller_portfolio.into(),
                asset: self.ticker2.into(),
                amount: 20 * UNIT,
              }],
              vec![
                our_portfolio.into(),
                caller_portfolio.into(),
              ],
            ).submit()?;

            Ok(())
        }

        #[ink(message)]
        /// Accept custody of a portfolio and give the caller some tokens.
        pub fn add_portfolio(&mut self, auth_id: u64, portfolio: PortfolioKind) -> Result<()> {
            self.ensure_initialized()?;
            // Get the caller's identity.
            let caller_did = self.get_caller_did()?;
            // Ensure the caller doesn't have a portfolio.
            self.ensure_no_portfolio(caller_did)?;

            let portfolio = PortfolioId::kind(caller_did, portfolio);
            let api = Api::new();
            // Accept authorization.
            api.call().portfolio().accept_portfolio_custody(auth_id).submit()?;
            // Check that we are the custodian.
            if !api.query().portfolio().portfolios_in_custody(self.did, portfolio.into())? {
              return Err(Error::InvalidPortfolioAuthorization);
            }
            // Save the caller's portfolio.
            self.portfolios.insert(caller_did, &portfolio);

            // Give the caller some funds.
            self.fund_caller()?;
            Ok(())
        }

        #[ink(message)]
        /// Allow the caller to withdrawal funds from the contract controlled portfolio.
        pub fn withdrawal(&mut self, ticker: Ticker, amount: Balance, dest: PortfolioKind) -> Result<()> {
            self.ensure_initialized()?;
            self.ensure_ticker(ticker)?;

            // Get the caller's identity.
            let caller_did = self.get_caller_did()?;
            let dest = PortfolioId::kind(caller_did, dest);

            // Ensure the caller has a portfolio.
            let caller_portfolio = self.ensure_has_portfolio(caller_did)?;

            let api = Api::new();
            // Move funds out of the contract controlled portfolio.
            api.call().portfolio().move_portfolio_funds(
              caller_portfolio.into(), // Contract controlled portfolio.
              dest.into(), // Caller controlled portfolio.
              vec![MovePortfolioItem {
                ticker: ticker.into(),
                amount,
                memo: None,
              }]).submit()?;
            Ok(())
        }

        #[ink(message)]
        /// Return the caller's portfolio custodianship back to them.
        pub fn withdrawal_all(&mut self) -> Result<()> {
            self.ensure_initialized()?;

            // Get the caller's identity.
            let caller_did = self.get_caller_did()?;

            // Ensure the caller has a portfolio.
            let portfolio = self.ensure_has_portfolio(caller_did)?;

            let api = Api::new();
            // Remove our custodianship.
            api.call().portfolio().quit_portfolio_custody(portfolio.into()).submit()?;
            // Remove the portfolio.
            self.portfolios.remove(caller_did);

            Ok(())
        }

        #[ink(message)]
        /// Trade.
        pub fn trade(&mut self, sell: Ticker, sell_amount: Balance, buy: Ticker, buy_amount: Balance) -> Result<()> {
            self.ensure_initialized()?;
            self.ensure_ticker(sell)?;
            self.ensure_ticker(buy)?;

            // Get the caller's identity.
            let caller_did = self.get_caller_did()?;

            // Ensure the caller has a portfolio.
            let caller_portfolio = self.ensure_has_portfolio(caller_did)?;

            let api = Api::new();
            // Use settlement to complete the trade.
            let our_portfolio = PortfolioId::default(self.did);
            api.call().settlement().add_and_affirm_instruction(
              self.venue.into(),
              SettlementType::SettleOnAffirmation,
              None,
              None,
              vec![Leg {
                from: caller_portfolio.into(),
                to: our_portfolio.into(),
                asset: sell.into(),
                amount: sell_amount,
              }, Leg {
                from: our_portfolio.into(),
                to: caller_portfolio.into(),
                asset: buy.into(),
                amount: buy_amount,
              }],
              vec![
                our_portfolio.into(),
                caller_portfolio.into(),
              ],
            ).submit()?;

            Ok(())
        }
    }
}
