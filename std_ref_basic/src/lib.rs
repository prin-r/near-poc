use borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::{env, near_bindgen, AccountId};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

static E9: u128 = 1_000_000_000;

macro_rules! zip {
    ($x: expr) => ($x);
    ($x: expr, $($y: expr), +) => (
        $x.iter().map(|v| v.clone()).zip(zip!($($y.clone()), +))
    )
}

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct StdReferenceBasic {
    pub refs: UnorderedMap<String, (u128, u64, u64)>,
    pub owner: AccountId,
}

#[near_bindgen]
impl StdReferenceBasic {
    #[init]
    pub fn new() -> Self {
        assert!(!env::state_exists(), "ALREADY_INITIALIZED");
        Self {
            refs: UnorderedMap::new(b"refs".to_vec()),
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

    pub fn get_refs(&self, symbol: String) -> Option<(u128, u64, u64)> {
        match &symbol[..] {
            "USD" => Some((E9, env::block_timestamp(), 0)),
            _ => self.refs.get(&symbol),
        }
    }

    pub fn get_reference_data(&self, base: String, quote: String) -> Option<(u128, u64, u64)> {
        match (self.get_refs(base.clone()), self.get_refs(quote.clone())) {
            (Some((br, bt, _)), Some((qr, qt, _))) => return Some((br * E9 * E9 / qr, bt, qt)),
            (None, Some(_)) => env::log(format!("REF_DATA_NOT_AVAILABLE_FOR: {}", base).as_bytes()),
            (Some(_), None) => {
                env::log(format!("REF_DATA_NOT_AVAILABLE_FOR: {}", quote).as_bytes())
            }
            _ => env::log(format!("REF_DATA_NOT_AVAILABLE_FOR: {} and {}", base, quote).as_bytes()),
        }
        None
    }

    pub fn get_reference_data_bulk(
        &self,
        bases: Vec<String>,
        quotes: Vec<String>,
    ) -> Option<Vec<(u128, u64, u64)>> {
        assert!(bases.len() == quotes.len(), "BAD_INPUT_LENGTH");
        bases
            .iter()
            .zip(quotes.iter())
            .map(|(b, q)| self.get_reference_data(b.clone(), q.clone()))
            .collect()
    }

    pub fn relay(
        &mut self,
        symbols: Vec<String>,
        rates: Vec<String>,
        resolve_times: Vec<u64>,
        request_ids: Vec<u64>,
    ) {
        assert!(env::predecessor_account_id() == self.get_owner(), "NOT_AN_OWNER");

        let len = symbols.len();
        assert!(rates.len() == len, "BAD_RATES_LENGTH");
        assert!(resolve_times.len() == len, "BAD_RESOLVE_TIMES_LENGTH");
        assert!(request_ids.len() == len, "BAD_REQUEST_IDS_LENGTH");

        for (s, (r, (rt, rid))) in zip!(&symbols, &rates, &resolve_times, &request_ids) {
            let rate_opt = r.parse::<u128>().ok();
            assert!(rate_opt != None, format!("FAIL_TO_PARSE_RATE_{}_FOR_{}", r, s));
            self.refs.insert(&s, &(rate_opt.unwrap(), rt, rid));
            env::log(format!("relay: {},{},{},{}", s, r, rt, rid).as_bytes());
        }
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
        let contract = StdReferenceBasic::new();

        // check state
        assert_eq!(bob(), contract.owner);
        assert_eq!(true, contract.refs.is_empty());

        // check owner using view function
        assert_eq!(bob(), contract.get_owner());
    }

    #[test]
    fn test_transfer_ownership() {
        let mut context = get_context();
        context.predecessor_account_id = bob();

        testing_env!(context);
        let mut contract = StdReferenceBasic::new();

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
        let mut contract = StdReferenceBasic::new();

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
    fn test_get_refs_usd() {
        let context = get_context();
        testing_env!(context.clone());
        let contract = StdReferenceBasic::new();

        assert_eq!(
            Some((E9, context.block_timestamp, 0)),
            contract.get_refs("USD".into())
        );
    }

    #[test]
    fn test_get_refs_symbol_not_found() {
        let context = get_context();
        testing_env!(context.clone());
        let contract = StdReferenceBasic::new();

        assert_eq!(None, contract.get_refs("BTC".into()));
    }

    #[test]
    fn test_get_reference_data_usd() {
        let context = get_context();
        testing_env!(context.clone());
        let contract = StdReferenceBasic::new();

        assert_eq!(
            Some((E9 * E9, context.block_timestamp, context.block_timestamp)),
            contract.get_reference_data("USD".into(), "USD".into())
        );
    }

    #[test]
    fn test_get_reference_data_not_found() {
        let context = get_context();
        testing_env!(context.clone());
        let contract = StdReferenceBasic::new();

        assert_eq!(
            None,
            contract.get_reference_data("BTC".into(), "USD".into())
        );
    }

    #[test]
    fn test_get_reference_data_bulk_usd() {
        let context = get_context();
        testing_env!(context.clone());
        let contract = StdReferenceBasic::new();

        assert_eq!(
            Some(vec![
                (E9 * E9, context.block_timestamp, context.block_timestamp),
                (E9 * E9, context.block_timestamp, context.block_timestamp)
            ]),
            contract.get_reference_data_bulk(
                vec!["USD".into(), "USD".into()],
                vec!["USD".into(), "USD".into()]
            )
        );
    }

    #[test]
    fn test_get_reference_data_bulk_not_found() {
        let context = get_context();
        testing_env!(context.clone());
        let contract = StdReferenceBasic::new();

        assert_eq!(
            None,
            contract.get_reference_data_bulk(
                vec!["BTC".into(), "USD".into()],
                vec!["USD".into(), "USD".into()]
            )
        );
    }

    #[test]
    fn test_relay_and_get_refs() {
        let mut context = get_context();
        context.predecessor_account_id = bob();

        testing_env!(context.clone());
        let mut contract = StdReferenceBasic::new();

        contract.relay(
            vec!["BTC".into(), "ETH".into()],
            vec!["111000000000".into(), "222000000000".into()],
            vec![333, 444],
            vec![555, 666],
        );

        assert_eq!(Some((111 * E9, 333, 555)), contract.get_refs("BTC".into()));
        assert_eq!(Some((222 * E9, 444, 666)), contract.get_refs("ETH".into()));
    }

    #[test]
    #[should_panic(expected = "NOT_AN_OWNER")]
    fn test_relay_fail_because_not_owner() {
        let mut context = get_context();
        context.predecessor_account_id = alice();

        testing_env!(context.clone());
        let mut contract = StdReferenceBasic::new();

        contract.relay(
            vec!["BTC".into(), "ETH".into()],
            vec!["111000000000".into(),"222000000000".into()],
            vec![333, 444],
            vec![555, 666],
        );
    }

    #[test]
    #[should_panic(expected = "NOT_AN_OWNER")]
    fn test_relay_fail_because_owner_has_changed() {
        let mut context = get_context();
        context.predecessor_account_id = bob();

        testing_env!(context.clone());
        let mut contract = StdReferenceBasic::new();

        // transfer ownership to alice
        contract.transfer_ownership(alice());

        contract.relay(
            vec!["BTC".into(), "ETH".into()],
            vec!["111000000000".into(),"222000000000".into()],
            vec![333, 444],
            vec![555, 666],
        );
    }

    #[test]
    fn test_relay_and_get_reference_data() {
        let mut context = get_context();
        context.predecessor_account_id = bob();

        testing_env!(context.clone());
        let mut contract = StdReferenceBasic::new();

        contract.relay(
            vec!["BTC".into(), "ETH".into()],
            vec!["111000000000".into(), "222000000000".into()],
            vec![333, 444],
            vec![555, 666],
        );

        assert_eq!(
            Some((111 * E9 * E9, 333, context.block_timestamp)),
            contract.get_reference_data("BTC".into(), "USD".into())
        );
        assert_eq!(
            Some((222 * E9 * E9, 444, context.block_timestamp)),
            contract.get_reference_data("ETH".into(), "USD".into())
        );
        assert_eq!(
            Some((E9 * E9 / 2, 333, 444)),
            contract.get_reference_data("BTC".into(), "ETH".into())
        );
        assert_eq!(
            Some((2 * E9 * E9, 444, 333)),
            contract.get_reference_data("ETH".into(), "BTC".into())
        );
    }

    #[test]
    fn test_relay_and_get_reference_data_bulk() {
        let mut context = get_context();
        context.predecessor_account_id = bob();

        testing_env!(context.clone());
        let mut contract = StdReferenceBasic::new();

        contract.relay(
            vec!["BTC".into(), "ETH".into()],
            vec!["111000000000".into(), "222000000000".into()],
            vec![333, 444],
            vec![555, 666],
        );

        assert_eq!(
            Some(vec![
                (111 * E9 * E9, 333, context.block_timestamp),
                (222 * E9 * E9, 444, context.block_timestamp),
                (E9 * E9 / 2, 333, 444),
                (2 * E9 * E9, 444, 333),
            ]),
            contract.get_reference_data_bulk(
                vec!["BTC".into(), "ETH".into(), "BTC".into(), "ETH".into()],
                vec!["USD".into(), "USD".into(), "ETH".into(), "BTC".into()]
            )
        );
    }
}
