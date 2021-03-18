use borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::{env, ext_contract, near_bindgen, AccountId, Promise};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[ext_contract(ext)]
pub trait StdRef {
    fn get_reference_data(&self, base: String, quote: String) -> Option<(u128, u64, u64)>;
    fn get_reference_data_bulk(
        &self,
        bases: Vec<String>,
        quotes: Vec<String>,
    ) -> Option<Vec<(u128, u64, u64)>>;
}

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct StdProxy {
    pub ref_: AccountId,
    pub owner: AccountId,
}

#[near_bindgen]
impl StdProxy {
    #[init]
    pub fn new(ref_: AccountId) -> Self {
        assert!(!env::state_exists(), "ALREADY_INITIALIZED");
        Self {
            ref_: ref_,
            owner: env::signer_account_id(),
        }
    }

    pub fn get_owner(&self) -> AccountId {
        self.owner.clone()
    }

    pub fn transfer_ownership(&mut self, new_owner: AccountId) {
        assert!(env::predecessor_account_id() == self.get_owner(), "NOT_AN_OWNER");
        env::log(format!("transfer ownership from {} to {}", self.owner, new_owner).as_bytes());
        self.owner = new_owner;
    }

    pub fn get_ref(&self) -> AccountId {
        self.ref_.clone()
    }

    pub fn set_ref(&mut self, new_ref: AccountId) {
        assert!(env::predecessor_account_id() == self.get_owner(), "NOT_AN_OWNER");
        env::log(format!("set ref from {} to {}", self.ref_, new_ref).as_bytes());
        self.ref_ = new_ref
    }

    pub fn get_reference_data(
        &mut self,
        base: String,
        quote: String,
    ) -> Promise {
        ext::get_reference_data(base, quote, &self.ref_, 0, 9 * env::prepaid_gas() / 10)
    }

    pub fn get_reference_data_bulk(
        &mut self,
        bases: Vec<String>,
        quotes: Vec<String>,
    ) -> Promise {
        ext::get_reference_data_bulk(bases, quotes, &self.ref_, 0, 9 * env::prepaid_gas() / 10)
    }
}

// use the attribute below for unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

    fn alice() -> AccountId {
        "alice.near".to_string()
    }

    fn bob() -> AccountId {
        "bob.near".to_string()
    }

    fn carol() -> AccountId {
        "carol.near".to_string()
    }

    fn std_basic() -> AccountId {
        "std_basic.near".to_string()
    }

    fn get_context() -> VMContext {
        VMContext {
            current_account_id: alice(),
            signer_account_id: bob(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id: carol(),
            input: vec![],
            block_index: 0,
            block_timestamp: 0,
            account_balance: 0,
            account_locked_balance: 0,
            storage_usage: 0,
            attached_deposit: 0,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view: false,
            output_data_receivers: vec![],
            epoch_height: 0,
        }
    }

    #[test]
    fn test_create_new_contract() {
        let context = get_context();
        testing_env!(context);
        let contract = StdProxy::new(std_basic());

        // check state
        assert_eq!(bob(), contract.owner);
        assert_eq!(std_basic(), contract.ref_);

        // check owner using view function
        assert_eq!(std_basic(), contract.get_ref());
    }

    #[test]
    fn test_transfer_ownership() {
        let mut context = get_context();
        context.predecessor_account_id = bob();

        testing_env!(context);
        let mut contract = StdProxy::new(std_basic());

        // check owner using view function
        assert_eq!(bob(), contract.get_owner());
        // transfer the ownership bob -> alice
        contract.transfer_ownership(alice());
        // check owner using view function
        assert_eq!(alice(), contract.get_owner());
    }

    #[test]
    #[should_panic(expected = "NOT_AN_OWNER")]
    fn test_transfer_ownership_fail() {
        let mut context = get_context();
        context.predecessor_account_id = bob();

        testing_env!(context);
        let mut contract = StdProxy::new(std_basic());

        // check owner using view function
        assert_eq!(bob(), contract.get_owner());
        // transfer the ownership bob -> alice
        contract.transfer_ownership(alice());
        // check owner using view function
        assert_eq!(alice(), contract.get_owner());

        // transfer the ownership alice -> bob
        // should fail because bob is not an owner anymore
        contract.transfer_ownership(alice());
    }

    #[test]
    fn test_set_ref() {
        let mut context = get_context();
        context.predecessor_account_id = bob();

        testing_env!(context);
        let mut contract = StdProxy::new(std_basic());

        assert_eq!(std_basic(), contract.get_ref());

        contract.set_ref(alice());

        assert_eq!(alice(), contract.get_ref());
    }

    #[test]
    #[should_panic(expected = "NOT_AN_OWNER")]
    fn test_set_ref_fail() {
        let mut context = get_context();
        context.predecessor_account_id = bob();

        testing_env!(context);
        let mut contract = StdProxy::new(std_basic());

        contract.transfer_ownership(carol());

        assert_eq!(std_basic(), contract.get_ref());

        contract.set_ref(alice());

        assert_eq!(alice(), contract.get_ref());
    }
}
