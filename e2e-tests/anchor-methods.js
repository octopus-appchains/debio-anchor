module.exports = {
  changeMethods: [
    'new',
    'set_owner',
    // protocol settings
    'change_minimum_validator_deposit',
    'change_minimum_delegator_deposit',
    'change_minimum_total_stake_price_for_booting',
    'change_maximum_market_value_percent_of_near_fungible_tokens',
    'change_maximum_market_value_percent_of_wrapped_appchain_token',
    'change_minimum_validator_count',
    'change_maximum_validator_count',
    'change_maximum_validators_per_delegator',
    'change_unlock_period_of_validator_deposit',
    'change_unlock_period_of_delegator_deposit',
    'change_maximum_era_count_of_unwithdrawn_reward',
    'change_maximum_era_count_of_valid_appchain_message',
    'change_validator_commission_percent',
    // appchain settings
    'set_rpc_endpoint',
    'set_subql_endpoint',
    'set_era_reward',
    // anchor settings
    'set_token_price_maintainer_account',
    // oct price
    'set_price_of_oct_token',
    // appchain lifecycle
    'go_booting',
    'go_live',
    // staking & delegating
    'decrease_stake',
    'unbond_stake',
    'decrease_delegation',
    'unbond_delegation',
    'withdraw_stake',
    // wrapped token
    "set_account_of_wrapped_appchain_token",
    "set_metadata_of_wrapped_appchain_token",
    "set_premined_balance_of_wrapped_appchain_token"
  ],
  viewMethods: [
    'get_owner',
    'get_anchor_settings',
    'get_appchain_settings',
    'get_protocol_settings',
    "get_oct_token",
    "get_wrapped_appchain_token",
    "get_near_fungible_tokens",
    'get_appchain_state',
    'get_anchor_status',
    'get_validator_set_info_of',
    'get_processing_status_of',
    'get_index_range_of_staking_history',
    'get_staking_history',
    'get_index_range_of_anchor_event',
    'get_anchor_event',
    'get_index_range_of_token_bridging_history',
    'get_token_bridging_history',
    'get_validator_list_of_era',
    'get_delegators_of_validator_in_era',
    'get_unbonded_stakes_of',
    'get_validator_rewards_of',
    'get_delegator_rewards_of',
    'get_storage_balance',
  ],
};
