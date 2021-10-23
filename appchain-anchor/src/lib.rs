mod anchor_event_histories;
mod anchor_viewer;
mod appchain_lifecycle;
mod near_fungible_tokens;
mod permissionless_actions;
mod settings_manager;
mod staking;
mod storage_key;
mod storage_migration;
mod sudo_actions;
pub mod types;
mod validator_set;
mod wrapped_appchain_token;

use near_contract_standards::upgrade::Ownable;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedSet};
use near_sdk::json_types::{U128, U64};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    assert_self, env, ext_contract, log, near_bindgen, AccountId, Balance, Gas, PromiseOrValue,
    PromiseResult, Timestamp,
};

pub use anchor_event_histories::AnchorEventHistories;
pub use anchor_viewer::AnchorViewer;
pub use appchain_lifecycle::AppchainLifecycleManager;
pub use near_fungible_tokens::NearFungibleTokenManager;
pub use permissionless_actions::*;
pub use settings_manager::*;
pub use staking::StakingManager;
pub use wrapped_appchain_token::WrappedAppchainTokenManager;

use near_fungible_tokens::NearFungibleTokens;
use staking::{StakingHistories, UnbondedStakeReference};
use storage_key::StorageKey;
use types::*;
use validator_set::{ValidatorSet, ValidatorSetHistories};

/// Constants for gas.
const T_GAS: u64 = 1_000_000_000_000;
const GAS_FOR_FT_TRANSFER_CALL: u64 = 60 * T_GAS;
const GAS_FOR_BURN_FUNGIBLE_TOKEN: u64 = 80 * T_GAS;
const GAS_FOR_MINT_FUNGIBLE_TOKEN: u64 = 80 * T_GAS;
/// Gas cap for function `complete_switching_era`.
const GAS_CAP_FOR_COMPLETE_SWITCHING_ERA: Gas = 180 * T_GAS;
/// The value of decimals value of USD.
const USD_DECIMALS_VALUE: Balance = 1_000_000;
/// The value of decimals value of OCT token.
const OCT_DECIMALS_VALUE: Balance = 1_000_000_000_000_000_000;
/// The seconds of a day.
const SECONDS_OF_A_DAY: u64 = 86400;
/// Multiple of nano seconds for a second.
const NANO_SECONDS_MULTIPLE: u64 = 1_000_000_000;
/// Storage deposit for NEP-141 token (in yocto)
const STORAGE_DEPOSIT_FOR_NEP141_TOEKN: Balance = 12_500_000_000_000_000_000_000;

#[ext_contract(ext_fungible_token)]
trait FungibleToken {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
    fn mint(&mut self, account_id: AccountId, amount: U128);
    fn burn(&mut self, account_id: AccountId, amount: U128);
}

#[ext_contract(ext_self)]
trait FungibleTokenContractResolver {
    /// Resolver for burning wrapped appchain token
    fn resolve_wrapped_appchain_token_burning(
        &mut self,
        sender_id_in_near: AccountId,
        receiver_id_in_appchain: String,
        amount: Balance,
    );
    /// Resolver for minting wrapped appchain token
    fn resolve_wrapped_appchain_token_minting(
        &mut self,
        sender_id_in_appchain: Option<String>,
        receiver_id_in_near: AccountId,
        amount: U128,
        appchain_message_nonce: u32,
    );
    /// Resolver for transfer NEAR fungible token
    fn resolve_fungible_token_transfer(
        &mut self,
        symbol: String,
        sender_id_in_appchain: String,
        receiver_id_in_near: AccountId,
        amount: U128,
        appchain_message_nonce: u32,
    );
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct AppchainAnchor {
    /// The id of corresponding appchain.
    appchain_id: AppchainId,
    /// The account id of appchain registry contract.
    appchain_registry: AccountId,
    /// The owner account id.
    owner: AccountId,
    /// The info of OCT token.
    oct_token: LazyOption<OctToken>,
    /// The info of wrapped appchain token in NEAR protocol.
    wrapped_appchain_token: LazyOption<WrappedAppchainToken>,
    /// The NEP-141 tokens data.
    near_fungible_tokens: LazyOption<NearFungibleTokens>,
    /// The history data of validator set.
    validator_set_histories: LazyOption<ValidatorSetHistories>,
    /// The validator set of the next era in appchain.
    /// This validator set is only for checking staking rules.
    next_validator_set: LazyOption<ValidatorSet>,
    /// The map of unwithdrawn validator rewards in eras, in unit of wrapped appchain token.
    unwithdrawn_validator_rewards: LookupMap<(u64, AccountId), Balance>,
    /// The map of unwithdrawn delegator rewards in eras, in unit of wrapped appchain token.
    unwithdrawn_delegator_rewards: LookupMap<(u64, AccountId, AccountId), Balance>,
    /// The map of unbonded stakes in eras.
    unbonded_stakes: LookupMap<AccountId, Vec<UnbondedStakeReference>>,
    /// The mapping for validators' accounts, from account id in the appchain to
    /// account id in NEAR protocol.
    validator_account_id_mapping: LookupMap<String, AccountId>,
    /// The custom settings for appchain.
    appchain_settings: LazyOption<AppchainSettings>,
    /// The anchor settings for appchain.
    anchor_settings: LazyOption<AnchorSettings>,
    /// The protocol settings for appchain anchor.
    protocol_settings: LazyOption<ProtocolSettings>,
    /// The state of the corresponding appchain.
    appchain_state: AppchainState,
    /// The staking history data happened in this contract.
    staking_histories: LazyOption<StakingHistories>,
    /// The anchor events data.
    anchor_event_histories: LazyOption<AnchorEventHistories>,
    /// The status of permissionless actions
    permissionless_actions_status: LazyOption<PermissionlessActionsStatus>,
}

impl Default for AppchainAnchor {
    fn default() -> Self {
        env::panic(b"The contract needs be initialized before use.")
    }
}

#[near_bindgen]
impl AppchainAnchor {
    #[init]
    pub fn new(
        appchain_id: AppchainId,
        appchain_registry: AccountId,
        oct_token: AccountId,
    ) -> Self {
        assert!(!env::state_exists(), "The contract is already initialized.");
        Self {
            appchain_id,
            appchain_registry,
            owner: env::predecessor_account_id(),
            oct_token: LazyOption::new(
                StorageKey::OctToken.into_bytes(),
                Some(&OctToken {
                    contract_account: oct_token,
                    price_in_usd: U128::from(0),
                }),
            ),
            wrapped_appchain_token: LazyOption::new(
                StorageKey::WrappedAppchainToken.into_bytes(),
                Some(&WrappedAppchainToken::default()),
            ),
            near_fungible_tokens: LazyOption::new(
                StorageKey::NearFungibleTokens.into_bytes(),
                Some(&NearFungibleTokens::new()),
            ),
            validator_set_histories: LazyOption::new(
                StorageKey::ValidatorSetHistories.into_bytes(),
                Some(&ValidatorSetHistories::new()),
            ),
            next_validator_set: LazyOption::new(
                StorageKey::NextValidatorSet.into_bytes(),
                Some(&ValidatorSet::new(u64::MAX)),
            ),
            unwithdrawn_validator_rewards: LookupMap::new(
                StorageKey::UnwithdrawnValidatorRewards.into_bytes(),
            ),
            unwithdrawn_delegator_rewards: LookupMap::new(
                StorageKey::UnwithdrawnDelegatorRewards.into_bytes(),
            ),
            unbonded_stakes: LookupMap::new(StorageKey::UnbondedStakes.into_bytes()),
            validator_account_id_mapping: LookupMap::new(
                StorageKey::LookupMapOfValidatorIdsInAppchain.into_bytes(),
            ),
            appchain_settings: LazyOption::new(
                StorageKey::AppchainSettings.into_bytes(),
                Some(&AppchainSettings {
                    chain_spec: String::new(),
                    raw_chain_spec: String::new(),
                    boot_nodes: String::new(),
                    rpc_endpoint: String::new(),
                    era_reward: U128::from(0),
                }),
            ),
            anchor_settings: LazyOption::new(
                StorageKey::AnchorSettings.into_bytes(),
                Some(&AnchorSettings {
                    token_price_maintainer_account: AccountId::new(),
                }),
            ),
            protocol_settings: LazyOption::new(
                StorageKey::ProtocolSettings.into_bytes(),
                Some(&ProtocolSettings::default()),
            ),
            appchain_state: AppchainState::Staging,
            staking_histories: LazyOption::new(
                StorageKey::StakingHistories.into_bytes(),
                Some(&StakingHistories::new()),
            ),
            anchor_event_histories: LazyOption::new(
                StorageKey::AnchorEventHistories.into_bytes(),
                Some(&AnchorEventHistories::new()),
            ),
            permissionless_actions_status: LazyOption::new(
                StorageKey::PermissionlessActionsStatus.into_bytes(),
                Some(&PermissionlessActionsStatus {
                    switching_era_number: Option::None,
                    distributing_reward_era_number: Option::None,
                }),
            ),
        }
    }
    // Assert that the contract called by the owner.
    fn assert_owner(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner,
            "Function can only be called by owner."
        );
    }
    // Assert that the contract called by a registered validator.
    fn assert_validator_id(&self, validator_id: &AccountId, next_validator_set: &ValidatorSet) {
        assert!(
            next_validator_set.validator_id_set.contains(validator_id)
                || next_validator_set.validators.contains_key(validator_id),
            "Validator id {} is not valid.",
            validator_id
        );
    }
    // Assert that the contract called by a registered validator.
    fn assert_delegator_id(
        &self,
        delegator_id: &AccountId,
        validator_id: &AccountId,
        next_validator_set: &ValidatorSet,
    ) {
        self.assert_validator_id(validator_id, next_validator_set);
        assert!(
            next_validator_set
                .validator_id_to_delegator_id_set
                .contains_key(validator_id),
            "Delegator id {} of validator {} is not valid.",
            delegator_id,
            validator_id
        );
        let delegator_id_set = next_validator_set
            .validator_id_to_delegator_id_set
            .get(validator_id)
            .unwrap();
        assert!(
            delegator_id_set.contains(delegator_id)
                || next_validator_set
                    .delegators
                    .contains_key(&(delegator_id.clone(), validator_id.clone())),
            "Delegator id {} of validator {} is not valid.",
            delegator_id,
            validator_id
        );
    }
    /// Set the price (in USD) of OCT token
    pub fn set_price_of_oct_token(&mut self, price: U128) {
        let anchor_settings = self.anchor_settings.get().unwrap();
        assert_eq!(
            env::predecessor_account_id(),
            anchor_settings.token_price_maintainer_account,
            "Only '{}' can call this function.",
            anchor_settings.token_price_maintainer_account
        );
        let mut oct_token = self.oct_token.get().unwrap();
        oct_token.price_in_usd = price;
        self.oct_token.set(&oct_token);
    }
    ///
    pub fn get_market_value_of_staked_oct_token(&self) -> U128 {
        U128::from(
            self.next_validator_set.get().unwrap().total_stake / OCT_DECIMALS_VALUE
                * self.oct_token.get().unwrap().price_in_usd.0,
        )
    }
}

#[near_bindgen]
impl Ownable for AppchainAnchor {
    fn get_owner(&self) -> AccountId {
        self.owner.clone()
    }

    fn set_owner(&mut self, owner: AccountId) {
        self.assert_owner();
        self.owner = owner;
    }
}

#[near_bindgen]
impl AppchainAnchor {
    /// Callback function for `ft_transfer_call` of NEP-141 compatible contracts
    pub fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        log!(
            "Deposit {} from @{} received. msg: {}",
            amount.0,
            &sender_id,
            msg
        );
        if env::predecessor_account_id().eq(&self.oct_token.get().unwrap().contract_account) {
            self.process_oct_deposit(sender_id, amount, msg)
        } else {
            self.process_near_fungible_token_deposit(
                env::predecessor_account_id(),
                sender_id,
                amount,
                msg,
            )
        }
    }
}

impl AppchainAnchor {
    ///
    pub fn append_anchor_event(&mut self, anchor_event: AnchorEvent) {
        let mut anchor_events = self.anchor_event_histories.get().unwrap();
        anchor_events.append(anchor_event);
        self.anchor_event_histories.set(&anchor_events);
    }
}
