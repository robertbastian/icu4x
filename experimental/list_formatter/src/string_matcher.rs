#[cfg(any(test, feature = "provider_transform_internals"))]
use crate::error::Error;
use alloc::borrow::Cow;
use core::lazy::OnceCell;
use icu_provider::yoke::{self, *};
use regex_automata::dfa::sparse::DFA;

#[derive(Clone, Debug, PartialEq, Yokeable, ZeroCopyFrom)]
#[cfg_attr(
    feature = "provider_serde",
    derive(serde::Deserialize, serde::Serialize),
    serde(transparent)
)]
pub(crate) struct StringMatcher<'data> {
    #[cfg_attr(feature = "provider_serde", serde(borrow))]
    dfa_bytes: Cow<'data, [u8]>,
    // Deserializing into a DFA can fail if the bytes are
    // corrupted. This memoizes the deserialized DFA.
    #[cfg_attr(feature = "provider_serde", serde(skip))]
    dfa: OnceCell<Option<DFA<&'data [u8]>>>,
}

impl<'data> StringMatcher<'data> {
    #[cfg(any(test, feature = "provider_transform_internals"))]
    pub(crate) fn new(pattern: &str) -> Result<Self, Error> {
        let dfa = {
            use regex_automata::{
                dfa::dense::{Builder, Config},
                SyntaxConfig,
            };
            let mut builder = Builder::new();
            builder
                .syntax(SyntaxConfig::new().case_insensitive(true))
                .configure(Config::new().anchored(true).minimize(true))
                .build(pattern)
                .map_err(Error::IllegalCondition)?
                .to_sparse()
                .map_err(Error::IllegalCondition)?
        };

        let dfa_bytes = Cow::Owned(dfa.to_bytes_little_endian());

        let dfa = OnceCell::new();
        dfa.set(Some(unsafe {
            DFA::from_bytes_unchecked(&*dfa_bytes).unwrap().0
        }))
        .unwrap();

        Ok(Self { dfa_bytes, dfa })
    }

    pub(crate) fn test(&self, string: &str) -> bool {
        use regex_automata::dfa::Automaton;
        self.dfa
            .get_or_init(|| DFA::from_bytes(&*self.dfa_bytes).ok().map(|(dfa, _)| dfa))
            .map(|dfa| dfa.find_earliest_fwd(string.as_bytes()).ok())
            // Option<Option<Option<HalfMatch>>
            .flatten()
            .flatten()
            .is_some()
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
