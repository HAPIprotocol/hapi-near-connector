use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::{AccountId, BorshStorageKey};
use serde::{Deserialize, Serialize};

use super::CategoryRisk;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum Category {
    // for all unspecified categories
    All,
    // HAPI returns 'None' when address wasn't find in database
    None,
    // Wallet service - custodial or mixed wallets
    WalletService,
    // Merchant service
    MerchantService,
    // Mining pool
    MiningPool,
    // Exchange
    Exchange,
    // DeFi application
    DeFi,
    // OTC Broker
    OTCBroker,
    // Cryptocurrency ATM
    ATM,
    // Gambling
    Gambling,
    // Illicit organization
    IllicitOrganization,
    // Mixer
    Mixer,
    // Darknet market or service
    DarknetService,
    // Scam
    Scam,
    // Ransomware
    Ransomware,
    // Theft - stolen funds
    Theft,
    // Counterfeit - fake assets
    Counterfeit,
    // Terrorist financing
    TerroristFinancing,
    // Sanctions
    Sanctions,
    // Child abuse and porn materials
    ChildAbuse,
}

pub type RiskScore = u8;

pub const MAX_RISK_LEVEL: RiskScore = 10;

#[derive(BorshStorageKey, BorshSerialize)]
enum StorageKey {
    AmlCategory,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct AML {
    pub(crate) account_id: AccountId,
    pub(crate) aml_conditions: UnorderedMap<Category, RiskScore>,
}

pub trait AmlManager {
    fn get_aml(&self) -> (&AccountId, Vec<CategoryRisk>);

    fn get_aml_conditions(&self) -> &UnorderedMap<Category, RiskScore>;

    fn update_account_id(&mut self, aml_account_id: AccountId);

    fn update_category(&mut self, category: Category, accepted_risk_score: RiskScore);
    fn remove_category(&mut self, category: Category);
}

impl AmlManager for AML {
    /// Returns the aml accountId and vector of added categories with accepted risk levels.
    ///
    /// # Examples
    ///
    /// ```
    /// use near_sdk::{AccountId, collections::UnorderedMap};
    /// use hapi_near_connector::aml::*;
    ///
    /// let aml_account: AccountId = AccountId::new_unchecked("aml".to_string());
    ///
    /// let aml: AML = AML::new(aml_account, MAX_RISK_LEVEL / 2);
    /// println!("{:?}", aml.get_aml());
    /// ```
    fn get_aml(&self) -> (&AccountId, Vec<(Category, RiskScore)>) {
        (
            &self.account_id,
            self.aml_conditions
                .iter()
                .map(|(id, acc)| (id, acc))
                .collect(),
        )
    }

    /// Returns reference to UnorderedMap of added categories with accepted risk levels.
    ///
    /// # Examples
    ///
    /// ```
    /// use near_sdk::{AccountId, collections::UnorderedMap};
    /// use hapi_near_connector::aml::*;
    ///
    /// let aml_account: AccountId = AccountId::new_unchecked("aml".to_string());
    ///
    /// let aml: AML = AML::new(aml_account, MAX_RISK_LEVEL / 2);
    /// println!("{:?}", aml.get_aml_conditions());
    /// ```
    fn get_aml_conditions(&self) -> &UnorderedMap<Category, RiskScore> {
        &self.aml_conditions
    }

    /// Updates account id of aml service.
    ///
    /// # Examples
    ///
    /// ```
    /// use near_sdk::{AccountId, collections::UnorderedMap};
    /// use hapi_near_connector::aml::*;
    ///
    /// let aml_account: AccountId = AccountId::new_unchecked("aml".to_string());
    ///
    /// let aml: AML = AML::new(aml_account, MAX_RISK_LEVEL / 2);
    ///
    /// let new_aml_account: AccountId = AccountId::new_unchecked("new_aml".to_string());
    /// aml.update_account_id(new_aml_account.clone());
    ///
    /// let (account_id, _) = aml.get_aml();
    /// assert_eq!(*account_id, new_aml_account);
    /// ```
    fn update_account_id(&mut self, account_id: AccountId) {
        self.account_id = account_id;
    }

    /// Updates or add category with accepted risk score to aml conditions.
    ///
    /// # Examples
    ///
    /// ```
    /// use near_sdk::{AccountId, collections::UnorderedMap};
    /// use hapi_near_connector::aml::*;
    ///
    /// let aml_account: AccountId = AccountId::new_unchecked("aml".to_string());
    ///
    /// let aml: AML = AML::new(aml_account, MAX_RISK_LEVEL / 2);
    ///
    /// aml.update_category(Category::Scam, 6);
    ///
    /// assert_eq!(aml.get_aml_conditions().get(&Category::Scam).unwrap(), 6);
    /// ```
    fn update_category(&mut self, category: Category, accepted_risk_score: RiskScore) {
        assert!(
            accepted_risk_score <= MAX_RISK_LEVEL,
            "ERR_RISK_SCORE_IS_INVALID"
        );
        assert!(accepted_risk_score > 0, "ERR_RISK_SCORE_IS_INVALID");
        self.aml_conditions.insert(&category, &accepted_risk_score);
    }

    /// Removes category from aml conditions.
    ///
    /// # Examples
    ///
    /// ```
    /// use near_sdk::{AccountId, collections::UnorderedMap};
    /// use hapi_near_connector::aml::*;
    ///
    /// let aml_account: AccountId = AccountId::new_unchecked("aml".to_string());
    ///
    /// let aml: AML = AML::new(aml_account, MAX_RISK_LEVEL / 2);
    ///
    /// aml.update_category(Category::Scam, 6);
    /// aml.remove_category(Category::Scam);
    ///
    /// assert!(aml.get_aml_conditions().get(&Category::Scam).is_none());
    /// ```
    fn remove_category(&mut self, category: Category) {
        assert!(category != Category::All);
        self.aml_conditions.remove(&category);
    }
}

impl AML {
    pub fn new(account_id: AccountId, accepted_risk_score: RiskScore) -> AML {
        let mut aml = Self {
            account_id,
            aml_conditions: UnorderedMap::new(StorageKey::AmlCategory),
        };
        aml.update_category(Category::All, accepted_risk_score);
        aml
    }
}
