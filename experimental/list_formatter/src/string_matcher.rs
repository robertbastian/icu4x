#[cfg(any(test, feature = "provider_transform_internals"))]
use crate::error::Error;
use alloc::borrow::Cow;
use core::cell::Cell;
use icu_provider::yoke::{self, *};

#[derive(Clone, Debug, PartialEq, Yokeable, ZeroCopyFrom)]
#[cfg_attr(
    feature = "provider_serde",
    derive(serde::Deserialize, serde::Serialize)
)]
#[yoke(cloning_zcf)]
#[serde(transparent)]
pub(crate) struct StringMatcher<'data> {
    #[cfg_attr(feature = "provider_serde", serde(borrow))]
    dfa_bytes: Cow<'data, [u8]>,
    // Deserializing into a DFA requires us to trust the bytes,
    // however unless we wrote them ourselves we cannot do that.
    // This field stores the verification result, so that we only
    // have to do that once.
    #[cfg_attr(feature = "provider_serde", serde(skip))]
    is_valid: Cell<Option<bool>>,
}

impl<'data> StringMatcher<'data> {
    #[cfg(any(test, feature = "provider_transform_internals"))]
    pub(crate) fn new(pattern: &str) -> Result<Self, Error> {
        use regex_automata::{
            dfa::dense::{Builder, Config, DFA},
            SyntaxConfig,
        };

        let mut builder = Builder::new();
        let dfa: DFA<Vec<u32>> = builder
            .syntax(SyntaxConfig::new().case_insensitive(true))
            .configure(Config::new().anchored(true).minimize(true))
            .build(pattern)
            .map_err(Error::IllegalCondition)?;

        Ok(Self {
            dfa_bytes: Cow::Owned(
                dfa.to_sparse()
                    .map_err(Error::IllegalCondition)?
                    .to_bytes_little_endian(),
            ),
            is_valid: Cell::new(None),
        })
    }

    pub(crate) fn test(&self, string: &str) -> bool {
        use regex_automata::dfa::sparse::DFA;
        use regex_automata::dfa::Automaton;

        if self.is_valid.get().is_none() {
            // This catches things like data corruption (which would lead to UB),
            // or wrong endianness.
            self.is_valid
                .set(Some(DFA::from_bytes(&*self.dfa_bytes).is_ok()));
        }
        self.is_valid.get().unwrap()
            && unsafe { DFA::from_bytes_unchecked(&*self.dfa_bytes).unwrap().0 }
                .find_earliest_fwd(string.as_bytes()) // Result<Option<HalfMatch>, Error>
                .ok() // Option<Option<HalfMatch>>
                .flatten() // Option<HalfMatch>
                .is_some() // bool
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_string_matcher() {
        let matcher = StringMatcher::new("abc.*").unwrap();
        assert!(!matcher.test("ab"));
        assert!(matcher.test("abc"));
        assert!(matcher.test("abcde"));
    }
}
