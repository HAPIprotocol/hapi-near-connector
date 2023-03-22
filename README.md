# HAPI Protocol

[HAPI Protocol] is a one-of-a-kind decentralized security protocol that prevents and interrupts any potential malicious activity within the blockchain space. HAPI Protocol works by leveraging both external and off-chain data as well as on-chain data accrued directly by HAPI Protocol and is publicly available.

# HAPI NEAR Proxy

It is a proxy [contract](https://github.com/HAPIprotocol/near-proxy-contract) used for replicating data from [HAPI Protocol] protocol main contract on the NEAR blockchain.

Address of the contract on the **testnet**

```
contract.hapi-test.testnet
```
Address of the contract on the **mainnet**

```
proxy.hapiprotocol.near
```

# HAPI NEAR Connector

This crate helps to implement [HAPI Protocol] in your smart contract on the NEAR blockchain. You can use it to request data from the proxy contract, and subsequent processing this data.

# Usage

### You need

1. Add [hapi-near-connector](https://crates.io/crates/hapi-near-connector) to dependencies of your project in Cargo.toml

2. Add use hapi-near-connector in your lib.rs file

3. Add field with AML struct to your Contract struct

4. Add `AML::new()` to init of your contract

5. Add cross-contract call to the method on which we need to use the protocol

6. Create a trait with a callback to handle response from HAPI Protocol NEAR proxy.

It is short description of steps you need to implement [HAPI Protocol] in you contract. For detailed information go to [example](#example-of-integration-into-the-ft-contract)

### The crate has an AML structure that stores

- account_id: AccountId - the address of the HAPI proxy [contract](https://github.com/HAPIprotocol/near-proxy-contract);
- pub aml_conditions: UnorderedMap<Category, RiskScore> - a map of categories and corresponding risk levels that you will add.

>Note
>
>If the risk level was not set for some categories, then the risk level for the category **All** is used.

## Methods
___________ 
  
- get_aml - Returns the aml accountId and vector of added categories with accepted risk levels.

- update_account_id - Updates account id of aml service.

- update_category - Updates or add a category with accepted risk score to aml conditions.

- remove_category - Removes category from aml conditions.

- assert_risk - Checks the category according to the set level of risk, if risk is higher than allowed it panics. If the risk level is not set for this category, it checks the All category.

- get_aml_conditions - Returns reference to UnorderedMap of added categories with accepted risk levels.

- check_risk -  Returns true if the address is risky or false if not.

## Integration into an existing contract

For integration into an existing contract do steps from [this list](#you-need).

And add a method that migrates your old contract struct to a new struct that includes [AML](#the-crate-has-an-aml-structure-that-stores) field.

Then you need to rebuild and redeploy the new wasm.

[Example](#example-integration-into-an-existing-contract)

## Example of integration into the FT contract
___________ 

1. Add [hapi-near-connector](https://crates.io/crates/hapi-near-connector) to dependencies of your project in Cargo.toml
```rust
[dependencies]
...
hapi-near-connector = "0.3.0"
```

2. Add use hapi_near_connector in your lib.rs file
```rust
use hapi_near_connector::aml::*;
```

3. Add field with AML struct to your Contract struct
```rust
pub struct Contract {
    token: FungibleToken,
    metadata: LazyOption<FungibleTokenMetadata>,
    owner_id: AccountId,
    aml: AML, // field added to use HAPI Protocol
}
```

4. Add "AML new" to init of your contract

>Note. Here we set the accepted risk level as MAX_RISK_LEVEL/2 i.e 10/2 = 5. 
```rust
aml: AML::new(aml_account_id, MAX_RISK_LEVEL / 2)
```

5. Add cross-contract call to the method on which we need to use the protocol. 

In this example we add cross-contract call to the ft_transfer method. So when user call ft_transfer a smart-contract request data about user then handle it and and ends the execution depending on the data from the HAPI Protocol.

* *ext_aml* - it's the connector's trait. 
* In get_address pass accountId you want to check. 
* *ext_self* - is the trait that will be created in the next step. 
* cb_ft_transfer - is the method for callback after aml.

```rust
fn ft_transfer(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
) -> Promise {
    assert_one_yocto();
    let sender: AccountId = predecessor_account_id();

    // it is a cross-contract call to the proxy contract
    ext_aml::ext(self.aml.get_account())
        .with_static_gas(AML_CHECK_GAS)
        .get_address(sender.clone()) // here we pass the address that needs to be checked
        .then(
            ext_self::ext(current_account_id())
                .with_static_gas(CALLBACK_AML_GAS)
                .cb_ft_transfer(sender, receiver_id, amount, memo), // here call method that handle response from proxy contract
        )
}
```

6. Create a trait with a callback and impl of it
```rust
#[ext_contract(ext_self)]
pub trait ExtContract {
    /// Callback after ft_transfer.
    fn cb_ft_transfer(
        &mut self,
        sender_id: AccountId,
        #[callback] category_risk: CategoryRisk,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
    );
}

#[near_bindgen]
impl ExtContract for Contract {
    #[private]
    fn cb_ft_transfer(
        &mut self,
        sender_id: AccountId,
        #[callback] category_risk: CategoryRisk,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
    ) {
        // here we check response from proxy contract, if you don't need to assert use "check_risk" function
        self.aml.assert_risk(category_risk);
        // and then you can to continue performing the function
        self.token.ft_transfer(sender_id, receiver_id, amount, memo)
    }
}
```

After that, your contract can already work with HAPI Protocol.

If you need to change the accepted risk level for *All* categories or add a new one, use *update_category*.


```rust
fn update_category(&mut self, category: Category, risk_score: RiskScore) {
    self.assert_owner();
    self.aml.update_category(category, risk_score);
}
```

Also, you can delete an added category. Then it will be evaluated in the *All* category.
>Note. You can't delete the category *All*.

```rust
fn remove_category(&mut self, category: Category) {
    self.assert_owner();
    self.aml.remove_category(category);
}
```

## Example integration into an existing contract

```rust
pub trait Migrations {
    fn add_hapi(aml_account_id: AccountId) -> Self;
}

#[near_bindgen]
impl Migrations for Contract {
    #[private]
    #[init(ignore_state)]
    #[allow(dead_code)]
    fn add_hapi(aml_account_id: AccountId) -> Self {
        #[derive(BorshDeserialize)]
        pub struct OldContract {
            token: FungibleToken,
            metadata: LazyOption<FungibleTokenMetadata>,
            owner_id: AccountId,
        }

        let old_contract: OldContract = env::state_read().expect("Old state doesn't exist");

        let mut aml = AML::new(aml_account_id, MAX_RISK_LEVEL / 2);

        // if you doesn't plan to add categories often you can do it right away
        aml.update_category(Category::Exchange, 4);

        Self {
            token: old_contract.token,
            metadata: old_contract.metadata,
            owner_id: old_contract.owner_id,
            aml,
        }
    }
}
```
# Alternative

Also you can integrate [HAPI Protocol] without this crate, you can find [example on Jumbo exchange](https://github.com/jumbo-exchange/contracts#hapi-protocol-integration).


[HAPI Protocol]: https://hapi-one.gitbook.io/hapi-protocol/