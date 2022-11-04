use pest::{
    iterators::{Pair, Pairs},
    Parser,
};

use std::str::FromStr;

#[derive(Parser)]
#[grammar = "parser/jsonpath.pest"]
struct JSONPathParser;

#[derive(Debug, PartialEq, Eq)]
pub enum Selector {
    Root,
    DotMemberName(String),
    Wildcard,
    ArrayIndex(isize),
    DecendantDotMemberName(String),
    DecendantWildcard,
    DecendantArrayIndex(isize),
    Union(Vec<UnionMember>),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum UnionMember {
    MemberName(String),
    ArrayIndex(isize),
}

pub fn parse(source: &str) -> Result<Vec<Selector>, String> {
    let pairs = match JSONPathParser::parse(Rule::jsonpath, source) {
        Ok(v) => v,
        Err(e) => return Err(format!("unable to parse: {}", e)),
    };
    parse_pairs(pairs)
}

fn parse_pairs(pairs: Pairs<Rule>) -> Result<Vec<Selector>, String> {
    pairs
        .into_iter()
        .filter(|p| p.as_rule() != Rule::EOI)
        .map(parse_pair)
        .collect()
}

macro_rules! inner {
    ($pair:expr,$fun:expr) => {
        $pair.into_inner().map($fun).next().unwrap()
    };
}

fn parse_pair(pair: Pair<Rule>) -> Result<Selector, String> {
    match pair.as_rule() {
        Rule::root => Ok(Selector::Root),
        Rule::selector => inner!(pair, parse_pair),
        Rule::dot_selector => inner!(pair, parse_dot_selector),
        Rule::dot_wildcard_selector => Ok(Selector::Wildcard),
        Rule::index_selector => inner!(pair, parse_index_selector),
        Rule::index_wildcard_selector => Ok(Selector::Wildcard),
        Rule::array_slice_selector => inner!(pair, parse_array_slice_selector),
        Rule::decendant_selector => inner!(pair, parse_decendant_selector),
        Rule::union_selector => parse_union_selector(pair),
        Rule::filter_selector => inner!(pair, parse_filter_selector),
        _ => unreachable!(),
    }
}

fn parse_dot_selector(pair: Pair<Rule>) -> Result<Selector, String> {
    Ok(Selector::DotMemberName(pair.as_str().to_owned()))
}

fn parse_index_selector(pair: Pair<Rule>) -> Result<Selector, String> {
    match pair.as_rule() {
        Rule::quoted_member_name => {
            let member_name = member_name_from_quoted(pair)?;
            Ok(Selector::DotMemberName(member_name))
        }
        Rule::element_index => {
            let array_index = array_index(pair)?;
            Ok(Selector::ArrayIndex(array_index))
        }
        _ => unreachable!(),
    }
}

fn parse_decendant_selector(pair: Pair<Rule>) -> Result<Selector, String> {
    // parse variants ignoring the decendant
    let aux = match pair.as_rule() {
        Rule::dot_member_name => parse_dot_selector(pair),
        Rule::index_selector => inner!(pair, parse_index_selector),
        Rule::wildcard | Rule::index_wildcard_selector => Ok(Selector::Wildcard),
        _ => unreachable!(),
    }?;
    // convert to decendant variant
    Ok(match aux {
        Selector::DotMemberName(m) => Selector::DecendantDotMemberName(m),
        Selector::ArrayIndex(i) => Selector::DecendantArrayIndex(i),
        Selector::Wildcard => Selector::DecendantWildcard,
        _ => unreachable!(),
    })
}

fn unimplemented<T>(pair: Pair<Rule>) -> Result<T, String> {
    Err(format!("unimplemented: {:}", pair.as_str()))
}

fn parse_union_selector(pair: Pair<Rule>) -> Result<Selector, String> {
    let cur = pair.into_inner();
    let mut children = vec![];
    for next in cur {
        let inner = next.into_inner().next().unwrap();
        match inner.as_rule() {
            Rule::quoted_member_name => {
                let member_name = member_name_from_quoted(inner)?;
                children.push(UnionMember::MemberName(member_name));
            }
            Rule::element_index => {
                let array_index = array_index(inner)?;
                children.push(UnionMember::ArrayIndex(array_index));
            }
            _ => unreachable!(),
        };
    }
    Ok(Selector::Union(children))
}

fn parse_array_slice_selector(pair: Pair<Rule>) -> Result<Selector, String> {
    unimplemented(pair)
}

fn parse_filter_selector(pair: Pair<Rule>) -> Result<Selector, String> {
    unimplemented(pair)
}

fn member_name_from_quoted(pair: Pair<Rule>) -> Result<String, String> {
    inner!(pair, |inner| {
        let s = inner.as_str();
        let mut chars = s.chars();
        let mut res = String::with_capacity(s.len());
        while let Some(c) = chars.next() {
            if c != '\\' {
                res.push(c);
                continue;
            }
            let escaped = match chars.next() {
                Some(cc) => cc,
                None => return Err(format!("invalid escape sequence: {}", s)),
            };
            let unescaped = match escaped {
                'b' => '\u{0008}',
                't' => '\u{0009}',
                'n' => '\u{000A}',
                'f' => '\u{000C}',
                'r' => '\u{000D}',
                '"' => '\u{0022}',
                '\'' => '\u{0027}',
                '/' => '\u{002F}',
                '\\' => '\u{005C}',
                'u' => {
                    let encoded: String = match chars.next_chunk::<4>() {
                        Ok(v) => v.iter().collect(),
                        Err(_) => return Err(format!("invalid unicode sequence: {}", s)),
                    };
                    let code_point = match u32::from_str_radix(&encoded, 16) {
                        Ok(v) => v,
                        Err(_) => return Err(format!("invalid unicode sequence: {}", s)),
                    };
                    match char::from_u32(code_point) {
                        Some(code_point_char) => code_point_char,
                        None => return Err(format!("invalid unicode sequence: {}", s)),
                    }
                }
                _ => return Err(format!("invalid escape sequence: {}", s)),
            };
            res.push(unescaped)
        }
        Ok(res)
    })
}

//TODO: check size
fn array_index(pair: Pair<Rule>) -> Result<isize, String> {
    match isize::from_str(pair.as_str()) {
        Ok(v) => Ok(v),
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_success_tests() {
        struct Test<'a> {
            input: &'a str,
            expect: Vec<Selector>,
        }
        vec![
            Test {
                input: "",
                expect: vec![Selector::Root],
            },
            Test {
                input: ".a",
                expect: vec![Selector::Root, Selector::DotMemberName("a".to_owned())],
            },
            Test {
                input: "$.ab",
                expect: vec![Selector::Root, Selector::DotMemberName("ab".to_owned())],
            },
            Test {
                input: "$",
                expect: vec![Selector::Root],
            },
            Test {
                input: "$..[*]",
                expect: vec![Selector::Root, Selector::DecendantWildcard],
            },
            Test {
                input: "$..*",
                expect: vec![Selector::Root, Selector::DecendantWildcard],
            },
            Test {
                input: "$[-1]",
                expect: vec![Selector::Root, Selector::ArrayIndex(-1)],
            },
            Test {
                input: "$[10]",
                expect: vec![Selector::Root, Selector::ArrayIndex(10)],
            },
            Test {
                input: r#"$["a"]"#,
                expect: vec![Selector::Root, Selector::DotMemberName("a".to_owned())],
            },
            Test {
                input: r#"$["\"a\""]"#,
                expect: vec![Selector::Root, Selector::DotMemberName("\"a\"".to_owned())],
            },
            Test {
                input: "$['\\'a\\'']",
                expect: vec![Selector::Root, Selector::DotMemberName("'a'".to_owned())],
            },
            Test {
                input: r#"$["\u263A"]"#,
                expect: vec![Selector::Root, Selector::DotMemberName("☺".to_owned())],
            },
            Test {
                input: "$.a.b",
                expect: vec![
                    Selector::Root,
                    Selector::DotMemberName("a".to_owned()),
                    Selector::DotMemberName("b".to_owned()),
                ],
            },
            Test {
                input: "$..a",
                expect: vec![
                    Selector::Root,
                    Selector::DecendantDotMemberName("a".to_owned()),
                ],
            },
            Test {
                input: "$['a']['b']..c[*]",
                expect: vec![
                    Selector::Root,
                    Selector::DotMemberName("a".to_owned()),
                    Selector::DotMemberName("b".to_owned()),
                    Selector::DecendantDotMemberName("c".to_owned()),
                    Selector::Wildcard,
                ],
            },
            Test {
                input: "$['a'][100][*]",
                expect: vec![
                    Selector::Root,
                    Selector::DotMemberName("a".to_owned()),
                    Selector::ArrayIndex(100),
                    Selector::Wildcard,
                ],
            },
            Test {
                input: "$['a','b',1]",
                expect: vec![
                    Selector::Root,
                    Selector::Union(vec![
                        UnionMember::MemberName("a".to_owned()),
                        UnionMember::MemberName("b".to_owned()),
                        UnionMember::ArrayIndex(1),
                    ]),
                ],
            },
        ]
        .iter()
        .for_each(|test| {
            let selectors =
                parse(test.input).unwrap_or_else(|e| panic!("error parsing {}: {}", test.input, e));
            assert_eq!(selectors, test.expect);
        })
    }
    #[test]
    fn parse_failure_tests() {
        vec![
            ".",
            "()",
            "$.",
            "$..",
            "..",
            "$.5",
            "$[abc]",
            "$[9999999999999999999999999999999999999999]",
            "$['1'",
            "$['1]",
            "$['1',]",
            "$[abc]",
            "$['a',99999999999999999999999999999999999999]",
            r#"$["\uXX"]"#,
            r#"$["\u001"]"#,
            r#"$["\uABCG"]"#,
        ]
        .iter()
        .for_each(|input| {
            parse(input).expect_err(&format!("expected error parsing {}", input));
        })
    }
}
