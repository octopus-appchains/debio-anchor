use crate::*;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap};
use near_sdk::{env, near_bindgen, AccountId, Balance, BlockHeight};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum TokenBridgingFact {
    /// The fact that a certain amount of wrapped appchain token is minted in its contract
    /// in NEAR protocol
    WrappedAppchainTokenMinted {
        request_id: String,
        /// The account id of receiver in NEAR protocol
        receiver_id: AccountId,
        amount: U128,
    },
    /// The fact that a certain amount of wrapped appchain token is burnt in its contract
    /// in NEAR protocol
    WrappedAppchainTokenBurnt {
        sender_id: AccountId,
        /// The id of receiver on the appchain
        receiver_id: String,
        amount: U128,
    },
    /// The fact that a certain amount of NEP-141 token has been locked in appchain anchor.
    NearFungibleTokenLocked {
        symbol: String,
        /// The account id of sender in NEAR protocol
        sender_id: AccountId,
        /// The id of receiver on the appchain
        receiver_id: String,
        amount: U128,
    },
    /// The fact that a certain amount of NEP-141 token has been unlocked and
    /// transfered from this contract to the receiver.
    NearFungibleTokenUnlocked {
        request_id: String,
        symbol: String,
        /// The account id of receiver in NEAR protocol
        receiver_id: AccountId,
        amount: U128,
    },
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenBridgingHistory {
    pub token_bridging_fact: TokenBridgingFact,
    pub block_height: BlockHeight,
    pub timestamp: Timestamp,
    pub index: U64,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct TokenBridgingHistories {
    /// The token bridging history data happened in this contract.
    histories: LookupMap<u64, TokenBridgingHistory>,
    /// The start index of valid token bridging history.
    start_index: u64,
    /// The end index of valid token bridging history.
    end_index: u64,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct OldAppchainAnchor {
    /// The id of corresponding appchain.
    pub appchain_id: AppchainId,
    /// The account id of appchain registry contract.
    pub appchain_registry: AccountId,
    /// The owner account id.
    pub owner: AccountId,
    /// The info of OCT token.
    pub oct_token: LazyOption<OctToken>,
    /// The info of wrapped appchain token in NEAR protocol.
    pub wrapped_appchain_token: LazyOption<WrappedAppchainToken>,
    /// The NEP-141 tokens data.
    pub near_fungible_tokens: LazyOption<NearFungibleTokens>,
    /// The history data of validator set.
    pub validator_set_histories: LazyOption<ValidatorSetHistories>,
    /// The validator set of the next era in appchain.
    /// This validator set is only for checking staking rules.
    pub next_validator_set: LazyOption<ValidatorSet>,
    /// The map of unwithdrawn validator rewards in eras, in unit of wrapped appchain token.
    pub unwithdrawn_validator_rewards: LookupMap<(u64, AccountId), Balance>,
    /// The map of unwithdrawn delegator rewards in eras, in unit of wrapped appchain token.
    pub unwithdrawn_delegator_rewards: LookupMap<(u64, AccountId, AccountId), Balance>,
    /// The map of unbonded stakes in eras.
    pub unbonded_stakes: LookupMap<AccountId, Vec<UnbondedStakeReference>>,
    /// The mapping for validators' accounts, from account id in the appchain to
    /// account id in NEAR protocol.
    pub validator_account_id_mapping: LookupMap<String, AccountId>,
    /// The custom settings for appchain.
    pub appchain_settings: LazyOption<AppchainSettings>,
    /// The anchor settings for appchain.
    pub anchor_settings: LazyOption<AnchorSettings>,
    /// The protocol settings for appchain anchor.
    pub protocol_settings: LazyOption<ProtocolSettings>,
    /// The state of the corresponding appchain.
    pub appchain_state: AppchainState,
    /// The staking history data happened in this contract.
    pub staking_histories: LazyOption<StakingHistories>,
    /// The token bridging histories data happened in this contract.
    pub token_bridging_histories: LazyOption<TokenBridgingHistories>,
    /// The anchor events data.
    pub anchor_events: LazyOption<AnchorEventHistories>,
    /// The status of permissionless actions
    pub permissionless_actions_status: LazyOption<PermissionlessActionsStatus>,
}

#[near_bindgen]
impl AppchainAnchor {
    #[init(ignore_state)]
    pub fn migrate_state() -> Self {
        // Deserialize the state using the old contract structure.
        let old_contract: OldAppchainAnchor = env::state_read().expect("Old state doesn't exist");
        // Verify that the migration can only be done by the owner.
        // This is not necessary, if the upgrade is done internally.
        assert_eq!(
            &env::predecessor_account_id(),
            &old_contract.owner,
            "Can only be called by the owner"
        );

        // Create the new contract using the data from the old contract.
        let new_contract = AppchainAnchor {
            appchain_id: old_contract.appchain_id,
            appchain_registry: old_contract.appchain_registry,
            owner: old_contract.owner,
            oct_token: old_contract.oct_token,
            wrapped_appchain_token: old_contract.wrapped_appchain_token,
            near_fungible_tokens: old_contract.near_fungible_tokens,
            validator_set_histories: old_contract.validator_set_histories,
            next_validator_set: old_contract.next_validator_set,
            unwithdrawn_validator_rewards: old_contract.unwithdrawn_validator_rewards,
            unwithdrawn_delegator_rewards: old_contract.unwithdrawn_delegator_rewards,
            unbonded_stakes: old_contract.unbonded_stakes,
            validator_account_id_mapping: old_contract.validator_account_id_mapping,
            appchain_settings: old_contract.appchain_settings,
            anchor_settings: old_contract.anchor_settings,
            protocol_settings: old_contract.protocol_settings,
            appchain_state: old_contract.appchain_state,
            staking_histories: old_contract.staking_histories,
            anchor_event_histories: LazyOption::new(
                StorageKey::AnchorEventHistories.into_bytes(),
                Some(&AnchorEventHistories::new()),
            ),
            permissionless_actions_status: old_contract.permissionless_actions_status,
        };

        new_contract
    }
}
