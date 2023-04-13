use std::fmt::Display;

use cosmwasm_std::{StdError, StdResult};
use cw_storage_plus::{Key, KeyDeserialize, Prefixer, PrimaryKey};

use crate::AbstractError;

use super::{account_trace::AccountTrace, AccountSequence};

/// Unique identifier for an account.
/// On each chain this is unique, but not across chains.
#[cosmwasm_schema::cw_serde]
pub struct AccountId {
    /// Chain seq of the chain that triggered the account creation
    /// `AccountTrace::Local` if the account was created locally
    trace: AccountTrace,
    /// Unique identifier for the account
    /// Account factory sequence number for the trace chain
    seq: AccountSequence,
}

impl Display for AccountId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.trace, self.seq)
    }
}

impl AccountId {
    pub fn new(seq: AccountSequence, trace: AccountTrace) -> Result<Self, AbstractError> {
        trace.verify()?;
        Ok(Self { seq, trace })
    }
    // used internally for testing
    pub(crate) const fn const_new(seq: AccountSequence, trace: AccountTrace) -> Self {
        Self { seq, trace }
    }
    pub fn seq(&self) -> AccountSequence {
        self.seq
    }
    pub fn trace(&self) -> &AccountTrace {
        &self.trace
    }
}

impl<'a> PrimaryKey<'a> for &'a AccountId {
    type Prefix = &'a AccountTrace;

    type SubPrefix = ();

    type Suffix = AccountSequence;

    type SuperSuffix = Self;

    fn key(&self) -> Vec<cw_storage_plus::Key> {
        let mut keys = self.trace.key();
        keys.extend(self.seq.key());
        keys
    }
}

impl<'a> Prefixer<'a> for &AccountId {
    fn prefix(&self) -> Vec<Key> {
        self.key()
    }
}

impl KeyDeserialize for &AccountId {
    type Output = AccountId;

    #[inline(always)]
    fn from_vec(mut value: Vec<u8>) -> StdResult<Self::Output> {
        let mut tu = value.split_off(2);
        let t_len = parse_length(&value)?;
        let u = tu.split_off(t_len);

        Ok(AccountId {
            seq: AccountSequence::from_vec(u)?,
            trace: AccountTrace::from(String::from_vec(tu)?),
        })
    }
}

#[inline(always)]
fn parse_length(value: &[u8]) -> StdResult<usize> {
    Ok(u16::from_be_bytes(
        value
            .try_into()
            .map_err(|_| StdError::generic_err("Could not read 2 byte length"))?,
    )
    .into())
}

//--------------------------------------------------------------------------------------------------
// Tests
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::{testing::mock_dependencies, Addr, Order};
    use cw_storage_plus::Map;

    mod key {
        use super::*;

        fn mock_key() -> AccountId {
            AccountId {
                seq: 1,
                trace: AccountTrace::Remote(vec!["bitcoin".to_string()]),
            }
        }

        fn mock_keys() -> (AccountId, AccountId, AccountId) {
            (
                AccountId {
                    seq: 1,
                    trace: AccountTrace::Local,
                },
                AccountId {
                    seq: 1,
                    trace: AccountTrace::Remote(vec![
                        "ethereum".to_string(),
                        "bitcoin".to_string(),
                    ]),
                },
                AccountId {
                    seq: 2,
                    trace: AccountTrace::Remote(vec![
                        "ethereum".to_string(),
                        "bitcoin".to_string(),
                    ]),
                },
            )
        }

        #[test]
        fn storage_key_works() {
            let mut deps = mock_dependencies();
            let key = mock_key();
            let map: Map<&AccountId, u64> = Map::new("map");

            map.save(deps.as_mut().storage, &key, &42069).unwrap();

            assert_eq!(map.load(deps.as_ref().storage, &key).unwrap(), 42069);

            let items = map
                .range(deps.as_ref().storage, None, None, Order::Ascending)
                .map(|item| item.unwrap())
                .collect::<Vec<_>>();

            assert_eq!(items.len(), 1);
            assert_eq!(items[0], (key, 42069));
        }

        #[test]
        fn composite_key_works() {
            let mut deps = mock_dependencies();
            let key = mock_key();
            let map: Map<(&AccountId, Addr), u64> = Map::new("map");

            map.save(
                deps.as_mut().storage,
                (&key, Addr::unchecked("larry")),
                &42069,
            )
            .unwrap();

            map.save(
                deps.as_mut().storage,
                (&key, Addr::unchecked("jake")),
                &69420,
            )
            .unwrap();

            let items = map
                .prefix(&key)
                .range(deps.as_ref().storage, None, None, Order::Ascending)
                .map(|item| item.unwrap())
                .collect::<Vec<_>>();

            assert_eq!(items.len(), 2);
            assert_eq!(items[0], (Addr::unchecked("jake"), 69420));
            assert_eq!(items[1], (Addr::unchecked("larry"), 42069));
        }

        #[test]
        fn partial_key_works() {
            let mut deps = mock_dependencies();
            let (key1, key2, key3) = mock_keys();
            let map: Map<&AccountId, u64> = Map::new("map");

            map.save(deps.as_mut().storage, &key1, &42069).unwrap();

            map.save(deps.as_mut().storage, &key2, &69420).unwrap();

            map.save(deps.as_mut().storage, &key3, &999).unwrap();

            let items = map
                .prefix(&AccountTrace::Remote(vec![
                    "ethereum".to_string(),
                    "bitcoin".to_string(),
                ]))
                .range(deps.as_ref().storage, None, None, Order::Ascending)
                .map(|item| item.unwrap())
                .collect::<Vec<_>>();

            assert_eq!(items.len(), 2);
            assert_eq!(items[0], (1, 69420));
            assert_eq!(items[1], (2, 999));
        }
    }
}