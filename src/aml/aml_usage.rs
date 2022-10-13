use near_sdk::{ext_contract, AccountId, Gas};

use super::{Category, RiskScore, AML};

pub const AML_CHECK_GAS: Gas = near_sdk::Gas(10_000_000_000_000);
pub const CALLBACK_AML_GAS: Gas = near_sdk::Gas(10_000_000_000_000);

pub type CategoryRisk = (Category, RiskScore);

#[ext_contract(ext_aml)]
pub trait ExtAmlContract {
    fn get_address(&self, address: AccountId) -> CategoryRisk;
}

impl AML {
    /// Checks the category according to the set level of risk. If the risk level is not set for this category, it checks the All category.
    pub fn assert_risk(&self, category_risk: CategoryRisk) {
        let (category, risk) = category_risk;
        if category != Category::None {
            let accepted_risk = match self.aml_conditions.get(&category) {
                Some(risk) => risk,
                None => self
                    .aml_conditions
                    .get(&Category::All)
                    .expect("ERR_NO_DEFAULT_CATEGORY"),
            };

            assert!(risk <= accepted_risk, "ERR_AML_NOT_ALLOWED");
        }
    }

    pub fn get_account(&self) -> AccountId {
        self.account_id.clone()
    }
}
