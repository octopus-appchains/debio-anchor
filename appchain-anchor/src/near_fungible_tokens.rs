use near_sdk::serde_json;

use crate::*;

pub trait FungibleTokenContractResolver {
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

#[derive(BorshDeserialize, BorshSerialize)]
pub struct NearFungibleTokens {
    /// The set of symbols of NEP-141 tokens.
    symbols: UnorderedSet<String>,
    /// The NEP-141 tokens data, mapped by the symbol of the token.
    tokens: LookupMap<String, NearFungibleToken>,
}

impl NearFungibleTokens {
    ///
    pub fn new() -> Self {
        Self {
            symbols: UnorderedSet::new(StorageKey::NearFungibleTokenSymbols.into_bytes()),
            tokens: LookupMap::new(StorageKey::NearFungibleTokensMap.into_bytes()),
        }
    }
    ///
    pub fn to_vec(&self) -> Vec<NearFungibleToken> {
        let symbols = self.symbols.to_vec();
        symbols
            .iter()
            .map(|symbol| self.tokens.get(symbol).unwrap())
            .collect::<Vec<NearFungibleToken>>()
    }
    ///
    pub fn contains(&self, symbol: &String) -> bool {
        self.symbols.contains(symbol)
    }
    ///
    pub fn get(&self, symbol: &String) -> Option<NearFungibleToken> {
        self.tokens.get(symbol)
    }
    ///
    pub fn get_by_contract_account(&self, account_id: &AccountId) -> Option<NearFungibleToken> {
        let symbols = self.symbols.to_vec();
        for symbol in symbols {
            let near_fungible_token = self.tokens.get(&symbol).unwrap();
            if near_fungible_token.contract_account.eq(account_id) {
                return Some(near_fungible_token);
            }
        }
        None
    }
    ///
    pub fn insert(&mut self, near_fungible_token: &NearFungibleToken) {
        self.symbols.insert(&near_fungible_token.metadata.symbol);
        self.tokens
            .insert(&near_fungible_token.metadata.symbol, near_fungible_token);
    }
    ///
    pub fn total_market_value(&self) -> Balance {
        let mut total_market_value: u128 = 0;
        let symbols = self.symbols.to_vec();
        symbols.iter().for_each(|symbol| {
            let near_fungible_token = self.tokens.get(&symbol).unwrap();
            total_market_value += near_fungible_token.locked_balance.0
                / u128::pow(10, u32::from(near_fungible_token.metadata.decimals))
                * near_fungible_token.price_in_usd.0
        });
        total_market_value
    }
    ///
    pub fn get_market_value_of(&self, symbol: &String, amount: u128) -> Balance {
        if let Some(near_fungible_token) = self.tokens.get(&symbol) {
            amount / u128::pow(10, u32::from(near_fungible_token.metadata.decimals))
                * near_fungible_token.price_in_usd.0
        } else {
            0
        }
    }
}

pub trait NearFungibleTokenManager {
    ///
    fn register_near_fungible_token(
        &mut self,
        symbol: String,
        name: String,
        decimals: u8,
        contract_account: AccountId,
        price: U128,
    );
    ///
    fn change_near_fungible_token_metadata(
        &mut self,
        symbol: String,
        name: String,
        decimals: u8,
        contract_account: AccountId,
    );
    ///
    fn set_price_of_near_fungible_token(&mut self, symbol: String, price: U128);
    ///
    fn open_bridging_of_near_fungible_token(&mut self, symbol: String);
    ///
    fn close_bridging_of_near_fungible_token(&mut self, symbol: String);
}

#[near_bindgen]
impl NearFungibleTokenManager for AppchainAnchor {
    //
    fn register_near_fungible_token(
        &mut self,
        symbol: String,
        name: String,
        decimals: u8,
        contract_account: AccountId,
        price: U128,
    ) {
        self.assert_owner();
        let mut near_fungible_tokens = self.near_fungible_tokens.get().unwrap();
        assert!(
            !near_fungible_tokens.contains(&symbol),
            "Token {} is already registered.",
            &symbol
        );
        near_fungible_tokens.insert(&NearFungibleToken {
            metadata: NearFungibleTokenMetadata {
                symbol,
                name,
                decimals,
            },
            contract_account,
            price_in_usd: price,
            locked_balance: U128::from(0),
            bridging_state: BridgingState::Active,
        });
        self.near_fungible_tokens.set(&near_fungible_tokens);
    }
    //
    fn change_near_fungible_token_metadata(
        &mut self,
        symbol: String,
        name: String,
        decimals: u8,
        contract_account: AccountId,
    ) {
        self.assert_owner();
        let mut near_fungible_tokens = self.near_fungible_tokens.get().unwrap();
        assert!(
            near_fungible_tokens.contains(&symbol),
            "Token {} is not registered.",
            &symbol
        );
        let mut near_fungible_token = near_fungible_tokens.get(&symbol).unwrap();
        near_fungible_token.metadata.name = name;
        near_fungible_token.metadata.decimals = decimals;
        near_fungible_token.contract_account = contract_account;
        near_fungible_tokens.insert(&near_fungible_token);
    }
    //
    fn set_price_of_near_fungible_token(&mut self, symbol: String, price: U128) {
        let anchor_settings = self.anchor_settings.get().unwrap();
        assert_eq!(
            env::predecessor_account_id(),
            anchor_settings.token_price_maintainer_account,
            "Only '{}' can call this function.",
            anchor_settings.token_price_maintainer_account
        );
        let mut near_fungible_tokens = self.near_fungible_tokens.get().unwrap();
        assert!(
            near_fungible_tokens.contains(&symbol),
            "Token {} is not registered.",
            &symbol
        );
        let mut near_fungible_token = near_fungible_tokens.get(&symbol).unwrap();
        near_fungible_token.price_in_usd = price;
        near_fungible_tokens.insert(&near_fungible_token);
    }
    //
    fn open_bridging_of_near_fungible_token(&mut self, symbol: String) {
        self.assert_owner();
        let mut near_fungible_tokens = self.near_fungible_tokens.get().unwrap();
        assert!(
            near_fungible_tokens.contains(&symbol),
            "Token {} is not registered.",
            &symbol
        );
        let mut near_fungible_token = near_fungible_tokens.get(&symbol).unwrap();
        near_fungible_token.bridging_state = BridgingState::Active;
        near_fungible_tokens.insert(&near_fungible_token);
    }
    //
    fn close_bridging_of_near_fungible_token(&mut self, symbol: String) {
        self.assert_owner();
        let mut near_fungible_tokens = self.near_fungible_tokens.get().unwrap();
        assert!(
            near_fungible_tokens.contains(&symbol),
            "Token {} is not registered.",
            &symbol
        );
        let mut near_fungible_token = near_fungible_tokens.get(&symbol).unwrap();
        near_fungible_token.bridging_state = BridgingState::Closed;
        near_fungible_tokens.insert(&near_fungible_token);
    }
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
enum NearFungibleTokenDepositMessage {
    BridgeToAppchain { receiver_id_in_appchain: String },
}

impl AppchainAnchor {
    //
    pub fn process_near_fungible_token_deposit(
        &mut self,
        predecessor_account_id: AccountId,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let mut near_fungible_tokens = self.near_fungible_tokens.get().unwrap();
        if let Some(mut near_fungible_token) =
            near_fungible_tokens.get_by_contract_account(&predecessor_account_id)
        {
            let deposit_message: NearFungibleTokenDepositMessage =
                match serde_json::from_str(msg.as_str()) {
                    Ok(msg) => msg,
                    Err(_) => {
                        log!(
                            "Invalid msg '{}' attached in `ft_transfer_call`. Return deposit.",
                            msg
                        );
                        return PromiseOrValue::Value(amount);
                    }
                };
            match deposit_message {
                NearFungibleTokenDepositMessage::BridgeToAppchain {
                    receiver_id_in_appchain,
                } => {
                    assert!(
                        AccountIdInAppchain::new(receiver_id_in_appchain.clone()).is_valid(),
                        "Invalid receiver id in appchain: '{}'. Return deposit.",
                        &receiver_id_in_appchain
                    );
                    let protocol_settings = self.protocol_settings.get().unwrap();
                    assert!(
                        near_fungible_tokens.total_market_value()
                            + near_fungible_tokens.get_market_value_of(
                                &near_fungible_token.metadata.symbol,
                                amount.0
                            )
                            <= self.get_market_value_of_staked_oct_token().0
                                * u128::from(
                                    protocol_settings
                                        .maximum_market_value_percent_of_near_fungible_tokens
                                )
                                / 100,
                        "Too much NEAR fungible token to lock. Return deposit."
                    );
                    near_fungible_token.locked_balance =
                        match near_fungible_token.locked_balance.0.checked_add(amount.0) {
                            Some(value) => U128::from(value),
                            None => panic!("Locked balance overflow. Return deposit."),
                        };
                    near_fungible_tokens.insert(&near_fungible_token);
                    self.append_anchor_event(AnchorEvent::NearFungibleTokenLocked {
                        symbol: near_fungible_token.metadata.symbol.clone(),
                        sender_id_in_near: sender_id,
                        receiver_id_in_appchain,
                        amount,
                    });
                    return PromiseOrValue::Value(0.into());
                }
            }
        }
        panic!(
            "Invalid deposit '{}' of unknown NEP-141 asset from '{}' received. Return deposit.",
            amount.0, sender_id,
        );
    }
    //
    pub fn unlock_near_fungible_token(
        &mut self,
        sender_id_in_appchain: String,
        symbol: String,
        receiver_id_in_near: AccountId,
        amount: U128,
        appchain_message_nonce: u32,
    ) {
        let near_fungible_tokens = self.near_fungible_tokens.get().unwrap();
        if let Some(near_fungible_token) = near_fungible_tokens.get(&symbol) {
            ext_fungible_token::ft_transfer(
                receiver_id_in_near.clone(),
                amount,
                None,
                &near_fungible_token.contract_account,
                1,
                GAS_FOR_FT_TRANSFER_CALL,
            )
            .then(ext_self::resolve_fungible_token_transfer(
                symbol,
                sender_id_in_appchain,
                receiver_id_in_near.clone(),
                amount,
                appchain_message_nonce,
                &env::current_account_id(),
                0,
                env::prepaid_gas() / 4,
            ));
        }
    }
}

#[near_bindgen]
impl FungibleTokenContractResolver for AppchainAnchor {
    //
    fn resolve_fungible_token_transfer(
        &mut self,
        symbol: String,
        sender_id_in_appchain: String,
        receiver_id_in_near: AccountId,
        amount: U128,
        appchain_message_nonce: u32,
    ) {
        assert_self();
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                self.append_anchor_event(AnchorEvent::NearFungibleTokenUnlocked {
                    symbol: symbol.clone(),
                    sender_id_in_appchain,
                    receiver_id_in_near,
                    amount,
                    appchain_message_nonce,
                });
                let mut near_fungible_tokens = self.near_fungible_tokens.get().unwrap();
                if let Some(mut near_fungible_token) = near_fungible_tokens.get(&symbol) {
                    near_fungible_token.locked_balance =
                        match near_fungible_token.locked_balance.0.checked_sub(amount.0) {
                            Some(value) => U128::from(value),
                            None => U128::from(0),
                        };
                    near_fungible_tokens.insert(&near_fungible_token);
                };
            }
            PromiseResult::Failed => {
                self.append_anchor_event(AnchorEvent::FailedToUnlockNearFungibleToken {
                    symbol: symbol.clone(),
                    sender_id_in_appchain,
                    receiver_id_in_near: receiver_id_in_near.clone(),
                    amount,
                    appchain_message_nonce,
                    reason: format!(
                        "Maybe the receiver account {} is not registered in '{}' token contract.",
                        &receiver_id_in_near, &symbol
                    ),
                });
            }
        }
    }
}
