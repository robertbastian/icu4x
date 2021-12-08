#[cfg(feature = "provider_serde")]
use crate::error::Error;
use alloc::borrow::Cow;
use alloc::string::String;
use icu_provider::yoke::{self, *};
use regex_automata::dfa::sparse::DFA;

#[derive(Clone, Debug, Yokeable, ZeroCopyFrom)]
// TODO: Store the actual DFA instead of their serializations. This requires ZCF and Yokeable on them.
pub(crate) enum StringMatcher<'data> {
    // Constructor-created or deserialized from JSON. Always owned, Cow is required for ZCF.
    FromPattern(Cow<'data, str>, Cow<'data, [u8]>),
    // Deserialized from bincode. Always borrowed.
    Precomputed(Cow<'data, [u8]>),
}

impl PartialEq for StringMatcher<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (StringMatcher::FromPattern(pattern1, _), StringMatcher::FromPattern(pattern2, _)) => {
                pattern1 == pattern2
            }
            (StringMatcher::Precomputed(bytes1), StringMatcher::FromPattern(_, bytes2)) => {
                bytes1 == bytes2
            }
            (StringMatcher::FromPattern(_, bytes1), StringMatcher::Precomputed(bytes2)) => {
                bytes1 == bytes2
            }
            (StringMatcher::Precomputed(bytes1), StringMatcher::Precomputed(bytes2)) => {
                bytes1 == bytes2
            }
        }
    }
}

#[cfg(feature = "provider_serde")]
impl serde::Serialize for StringMatcher<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match self {
            StringMatcher::FromPattern(regex, _) if serializer.is_human_readable() => {
                regex.serialize(serializer)
            }
            StringMatcher::FromPattern(_, dfa_bytes) => dfa_bytes.serialize(serializer),
            StringMatcher::Precomputed(dfa_bytes) if !serializer.is_human_readable() => {
                dfa_bytes.serialize(serializer)
            }
            _ => {
                use serde::ser::Error;
                Err(S::Error::custom(
                    "Cannot serialize a deserialized bincode StringMatcher to JSON.",
                ))
            }
        }
    }
}

#[cfg(feature = "provider_serde")]
impl<'de: 'data, 'data> serde::Deserialize<'de> for StringMatcher<'data> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            StringMatcher::new(<&str>::deserialize(deserializer)?).map_err(|e| {
                use alloc::string::ToString;
                use serde::de::Error;
                D::Error::custom(e.to_string())
            })
        } else {
            if cfg!(target_endian = "big") {
                // TODO: Convert LE to BE. For now we just behave like the
                // accept-nothing DFA on BE systems.
                return Ok(StringMatcher::Precomputed(Cow::Borrowed(&[])));
            }

            let bytes = <Cow<'de, [u8]>>::deserialize(deserializer)?;
            DFA::from_bytes(&bytes).map_err(|e| {
                use serde::de::Error;
                D::Error::custom(alloc::format!("Invalid DFA bytes: {}", e))
            })?;
            Ok(StringMatcher::Precomputed(bytes))
        }
    }
}

impl<'data> StringMatcher<'data> {
    #[cfg(feature = "provider_serde")]
    pub(crate) fn new(pattern: &str) -> Result<Self, Error> {
        use regex_automata::{
            dfa::dense::{Builder, Config},
            SyntaxConfig,
        };
        let mut builder = Builder::new();
        let dfa = builder
            .syntax(SyntaxConfig::new().case_insensitive(true))
            .configure(Config::new().anchored(true).minimize(true))
            .build(pattern)
            .map_err(Error::IllegalCondition)?;

        let sparse_dfa = dfa.to_sparse().map_err(Error::IllegalCondition)?;

        Ok(Self::FromPattern(
            Cow::Owned(String::from(pattern)),
            Cow::Owned(sparse_dfa.to_bytes_little_endian()),
        ))
    }

    pub(crate) fn test(&self, string: &str) -> bool {
        #[cfg(target_endian = "big")]
        return false;

        use regex_automata::dfa::Automaton;

        let dfa = match self {
            StringMatcher::FromPattern(_, dfa_bytes) => unsafe {
                // This is safe (and Ok) because we created these bytes ourselves
                DFA::from_bytes_unchecked(&dfa_bytes).unwrap().0
            },
            StringMatcher::Precomputed(dfa_bytes) => unsafe {
                // This is safe (and Ok) because we validated the bytes during deserialization
                DFA::from_bytes_unchecked(dfa_bytes).unwrap().0
            },
        };
        matches!(dfa.find_earliest_fwd(string.as_bytes()), Ok(Some(_)))
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

    #[test]
    fn test_postcard_serialization() {
        let matcher = StringMatcher::new("abc*").unwrap();

        let mut bytes = postcard::to_stdvec(&matcher).unwrap();
        assert_eq!(
            postcard::from_bytes::<StringMatcher>(&bytes).unwrap(),
            matcher
        );

        // A corrupted byte leads to an error
        bytes[17] ^= 255;
        assert!(postcard::from_bytes::<StringMatcher>(&bytes).is_err());
        bytes[17] ^= 255;
    
        // An extra byte leads to an error
        bytes.insert(123, 40);
        assert!(postcard::from_bytes::<StringMatcher>(&bytes).is_err());
        bytes.remove(123);

        // Missing bytes lead to an error
        assert!(postcard::from_bytes::<StringMatcher>(&bytes[0..bytes.len()-5]).is_err());
    }

    #[test]
    fn test_json_serialization() {
        let matcher = StringMatcher::new("abc*").unwrap();

        let json = serde_json::to_string(&matcher).unwrap();
        assert_eq!(
            serde_json::from_str::<StringMatcher>(&json).unwrap(),
            matcher
        );
        assert!(serde_json::from_str::<StringMatcher>(&".*[").is_err());
    }
}
