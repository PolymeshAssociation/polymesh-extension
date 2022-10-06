#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use ink_lang as ink;
use ink_env::Environment;

#[cfg(feature = "type_info")]
use scale_info::TypeInfo;

use scale::{Decode, Encode};
use alloc::vec::Vec;

pub const TICKER_LEN: usize = 12;

pub type Ticker = [u8; TICKER_LEN];

/// A wrapper for a token name.
#[derive(Clone, Decode, Encode)]
#[cfg_attr(feature = "type_info", derive(TypeInfo))]
pub struct AssetName(pub Vec<u8>);

/// The ID of a custom asset type.
#[derive(Clone, Copy, Decode, Encode)]
#[cfg_attr(feature = "type_info", derive(TypeInfo))]
pub struct CustomAssetTypeId(pub u32);

/// The type of security represented by a token.
#[derive(Clone, Copy, Decode, Encode)]
#[cfg_attr(feature = "type_info", derive(TypeInfo))]
pub enum AssetType {
    /// Common stock - a security that represents ownership in a corporation.
    EquityCommon,
    /// Preferred stock. Preferred stockholders have a higher claim to dividends or asset
    /// distribution than common stockholders.
    EquityPreferred,
    /// Commodity - a basic good used in commerce that is interchangeable with other commodities of
    /// the same type.
    Commodity,
    /// Fixed income security - an investment that provides a return in the form of fixed periodic
    /// interest payments and the eventual return of principal at maturity. Examples: bonds,
    /// treasury bills, certificates of deposit.
    FixedIncome,
    /// Real estate investment trust - a company that owns, operates, or finances income-producing
    /// properties.
    REIT,
    /// Investment fund - a supply of capital belonging to numerous investors used to collectively
    /// purchase securities while each investor retains ownership and control of his own shares.
    Fund,
    /// Revenue share partnership agreement - a document signed by all partners in a partnership
    /// that has procedures when distributing business profits or losses.
    RevenueShareAgreement,
    /// Structured product, aka market-linked investment - a pre-packaged structured finance
    /// investment strategy based on a single security, a basket of securities, options, indices,
    /// commodities, debt issuance or foreign currencies, and to a lesser extent, derivatives.
    StructuredProduct,
    /// Derivative contract - a contract between two parties for buying or selling a security at a
    /// predetermined price within a specific time period. Examples: forwards, futures, options or
    /// swaps.
    Derivative,
    /// Anything else.
    Custom(CustomAssetTypeId),
    /// Stablecoins are cryptocurrencies designed to minimize the volatility of the price of the stablecoin,
    /// relative to some "stable" asset or basket of assets.
    /// A stablecoin can be pegged to a cryptocurrency, fiat money, or to exchange-traded commodities.
    StableCoin,
}

impl Default for AssetType {
    fn default() -> Self {
        Self::EquityCommon
    }
}

/// A wrapper for a funding round name.
#[derive(Clone, Decode, Encode)]
#[cfg_attr(feature = "type_info", derive(TypeInfo))]
pub struct FundingRoundName(pub Vec<u8>);

/// Implementation of common asset identifiers.
/// https://www.cusip.com/identifiers.html.
#[derive(Clone, Copy, Decode, Encode)]
#[cfg_attr(feature = "type_info", derive(TypeInfo))]
pub enum AssetIdentifier {
    /// Universally recognized identifier for financial instruments.
    /// Example: Amazon.com Inc - Common Stock
    /// ISSUER ISSUE CHECK CUSIP
    /// 023135 10    6     023135106
    CUSIP([u8; 9]),
    /// The CUSIP International Numbering System.
    /// Example: Abingdon Capital PLC - Shares
    /// COUNTRY CODE ISSUER ISSUE CHECK CINS
    /// G            0052B  10    5     G0052B105
    CINS([u8; 9]),
    /// The International Securities Identification Number.
    /// Example:
    /// COUNTRY CODE LOCAL IDENTIFIER CHECK ISIN
    /// CA           008911703        4     CA0089117034
    ISIN([u8; 12]),
    /// The Legal Entity Identifier.
    /// Example: Philadelphia Cheesesteak Company
    /// LOU PREFIX ENTITY INDENTIFIER VERIFICATION ID LEI
    /// 5493       00SAMIRN1R27UP     42              549300SAMIRN1R27UP42
    LEI([u8; 20]),
    /// Financial Instrument Global Identifier https://www.omg.org/figi/index.htm.
    /// Example: Alphabet Inc - Common Stock
    /// BBG013V1S0T3
    FIGI([u8; 12]),
}

#[ink::chain_extension]
pub trait PolymeshRuntime {
    type ErrorCode = PolymeshRuntimeErr;

    #[ink(extension = 0x00_1A_00_00, returns_result = false)]
    fn register_ticker(ticker: Ticker);

    #[ink(extension = 0x00_1A_01_00, returns_result = false)]
    fn accept_ticker_transfer(auth_id: u64);

    #[ink(extension = 0x00_1A_02_00, returns_result = false)]
    fn accept_asset_ownership_transfer(auth_id: u64);

    #[ink(extension = 0x00_1A_03_00, returns_result = false)]
    fn create_asset(
      name: AssetName,
      ticker: Ticker,
      divisible: bool,
      asset_type: AssetType,
      identifiers: Vec<AssetIdentifier>,
      funding_round: Option<FundingRoundName>,
      disable_iu: bool,
    );

    #[ink(extension = 0x00_1A_11_00, returns_result = false)]
    fn register_custom_asset_type(ty: Vec<u8>);
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "type_info", derive(TypeInfo))]
pub enum PolymeshRuntimeErr {
    Unknown,
}

impl ink_env::chain_extension::FromStatusCode for PolymeshRuntimeErr {
    fn from_status_code(status_code: u32) -> Result<(), Self> {
        match status_code {
            0 => Ok(()),
            1 => Err(Self::Unknown),
            _ => panic!("encountered unknown status code"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolymeshEnvironment {}

impl Environment for PolymeshEnvironment {
    const MAX_EVENT_TOPICS: usize =
        <ink_env::DefaultEnvironment as Environment>::MAX_EVENT_TOPICS;

    type AccountId = <ink_env::DefaultEnvironment as Environment>::AccountId;
    type Balance = <ink_env::DefaultEnvironment as Environment>::Balance;
    type Hash = <ink_env::DefaultEnvironment as Environment>::Hash;
    type BlockNumber = <ink_env::DefaultEnvironment as Environment>::BlockNumber;
    type Timestamp = <ink_env::DefaultEnvironment as Environment>::Timestamp;

    type ChainExtension = PolymeshRuntime;
}
