// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

use icu_properties::PropertyNamesLong;
use std::char;

struct TestContentIterator<LineIterator>(LineIterator);

#[allow(unused)]
struct TestData {
    original_line: String,
    chars: Vec<char>,
    break_result_utf8: Vec<usize>,
    break_result_utf16: Vec<usize>,
    break_result_latin1: Option<Vec<usize>>,
}

impl TestContentIterator<core::str::Split<'static, char>> {
    pub fn new(file: &'static str) -> Self {
        Self(file.split('\n'))
    }
}

impl<LineIterator: Iterator> Iterator for TestContentIterator<LineIterator>
where
    LineIterator::Item: Into<String>,
{
    type Item = TestData;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let line: String = self.0.next()?.into();
            if line.is_empty() {
                // EOF
                return None;
            }
            if line.starts_with('#') {
                // Comment
                continue;
            }

            let mut r = line.split('#');
            let r = r.next();
            let v = r.unwrap().split_ascii_whitespace();
            let mut chars = Vec::new();
            let mut break_result_utf8 = Vec::new();
            let mut break_result_utf16 = Vec::new();
            let mut break_result_latin1 = Vec::new();

            let mut utf8_len = 0;
            let mut u16_len = 0;

            let mut ascii_only = true;
            for (count, item) in v.enumerate() {
                if count % 2 == 1 {
                    let ch = char::from_u32(u32::from_str_radix(item, 16).unwrap()).unwrap();
                    chars.push(ch);

                    utf8_len += ch.len_utf8();
                    u16_len += 1 + (ch as u32 > 0xFFFF) as usize;

                    if ch as u32 >= 0x100 {
                        ascii_only = false;
                    }
                } else if item != "\u{00d7}" {
                    assert_eq!(item, "\u{00f7}");
                    break_result_utf8.push(utf8_len);
                    break_result_utf16.push(u16_len);
                    break_result_latin1.push(chars.len());
                }
            }
            return Some(Self::Item {
                original_line: line,
                chars,
                break_result_utf8,
                break_result_utf16,
                break_result_latin1: if ascii_only {
                    Some(break_result_latin1)
                } else {
                    None
                },
            });
        }
    }
}

fn line_break_test(file: &'static str) {
    let test_iter = TestContentIterator::new(file);
    for (i, test) in test_iter.enumerate() {
        let s: String = test.chars.into_iter().collect();
        let iter = unicode_linebreak::linebreaks(&s).map(|(idx, _)| idx);
        let result: Vec<usize> = iter.collect();
        if result != test.break_result_utf8 {
            use icu_properties::{
                CodePointMapData,
                props::{GeneralCategory, LineBreak},
            };
            let lb = CodePointMapData::<LineBreak>::new();
            let lb_name = PropertyNamesLong::<LineBreak>::new();
            let gc = CodePointMapData::<GeneralCategory>::new();
            let gc_name = PropertyNamesLong::<GeneralCategory>::new();

            let mut iter = unicode_linebreak::linebreaks(&s).map(|(idx, _)| idx);
            // TODO(egg): It would be really nice to have Name here.
            println!("  | A | E | Code pt. | Line_Break         | General_Category | Literal");
            for (i, c) in s.char_indices() {
                let expected_break = test.break_result_utf8.contains(&i);
                let actual_break = result.contains(&i);
                if actual_break {
                    iter.next();
                }
                println!(
                    "{}| {} | {} | {:>8} | {:>18} | {:>18} | {}",
                    if actual_break != expected_break {
                        "😭"
                    } else {
                        "  "
                    },
                    if actual_break { "÷" } else { "×" },
                    if expected_break { "÷" } else { "×" },
                    format!("{:04X}", c as u32),
                    lb_name
                        .get(lb.get(c))
                        .unwrap_or(&format!("{:?}", lb.get(c))),
                    gc_name
                        .get(gc.get(c))
                        .unwrap_or(&format!("{:?}", gc.get(c))),
                    c
                )
            }
            println!("Test case #{i}");
            panic!()
        }
    }
}

#[test]
fn run_line_break_test() {
    line_break_test(include_str!("testdata/LineBreakTest_15.0.txt"));
}

#[test]
fn run_line_break_extra_test() {
    line_break_test(include_str!("testdata/LineBreakExtraTest_15.0.txt"));
}

#[test]
fn run_line_break_random_test() {
    line_break_test(include_str!("testdata/LineBreakRandomTest.txt"));
}
