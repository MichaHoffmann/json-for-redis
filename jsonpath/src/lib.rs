#![feature(iter_next_chunk)]

#[macro_use]
extern crate pest_derive;
extern crate pest;

pub mod parser;

use serde_json::Value;

use std::rc::Rc;

pub fn get<'a>(path: &str, val: &'a Value) -> Result<Vec<&'a Value>, String> {
    let selectors = parser::parse(path)?;
    if selectors == vec![parser::Selector::Root] {
        return Ok(vec![val]);
    }
    let paths = matches(selectors, val)?;
    let mut res = vec![];
    paths.into_iter().for_each(|path| {
        let mut segments = path.into_iter().peekable();
        let mut cur = val;
        while let Some(segment) = segments.next() {
            cur = match segment {
                PathSegment::MemberName(k) => {
                    let object = cur.as_object().unwrap();
                    if segments.peek().is_none() {
                        res.push(object.get(&k).unwrap());
                        break;
                    }
                    object.get(&k).unwrap()
                }
                PathSegment::ArrayIndex(i) => {
                    let array = cur.as_array().unwrap();
                    if segments.peek().is_none() {
                        res.push(array.get(i).unwrap());
                        break;
                    }
                    array.get(i).unwrap()
                }
            };
        }
    });
    Ok(res)
}

// For the mutating commands note that parents match before their children.
// To avoid setting a value in the parent we abort when we see that the prefix
// of a path was already mutated since that invalidates the subsequent match.

pub fn set(path: &str, val: &Value, to: &Value) -> Result<Value, String> {
    let mut selectors = parser::parse(path)?;
    if selectors == vec![parser::Selector::Root] {
        return Ok(to.clone());
    }

    // When the last element is a DotMemberName selector we also set it. To do this
    // we need to match to the second to last element and if we did so and this
    // element is an object we add/update that key.
    let maybe_last_selector = match selectors.last().unwrap() {
        parser::Selector::DotMemberName(_) => selectors.pop(),
        _ => None,
    };

    let mut res = val.clone();
    if selectors == vec![parser::Selector::Root] {
        if let Some(parser::Selector::DotMemberName(k)) = &maybe_last_selector {
            res.as_object_mut()
                .and_then(|obj| obj.insert(k.clone(), to.clone()));
        }
        return Ok(res);
    }

    let paths = matches(selectors, val)?;

    let mut prefix = vec![];
    paths.into_iter().for_each(|path| {
        if !prefix.is_empty() && path.starts_with(&prefix) {
            return;
        }
        prefix = path.clone();
        let mut segments = path.into_iter().peekable();
        let mut cur = &mut res;
        while let Some(segment) = segments.next() {
            cur = match segment {
                PathSegment::MemberName(k) => {
                    if let Some(object) = cur.as_object_mut() {
                        if segments.peek().is_none() {
                            if let Some(parser::Selector::DotMemberName(nk)) = &maybe_last_selector
                            {
                                object.get_mut(&k).and_then(|nested| {
                                    nested
                                        .as_object_mut()
                                        .and_then(|matched| matched.insert(nk.clone(), to.clone()))
                                });
                                break;
                            } else {
                                object.insert(k.clone(), to.clone());
                            }
                        }
                        object.get_mut(&k).unwrap()
                    } else {
                        break;
                    }
                }
                PathSegment::ArrayIndex(i) => {
                    if let Some(array) = cur.as_array_mut() {
                        if segments.peek().is_none() {
                            if let Some(parser::Selector::DotMemberName(nk)) = &maybe_last_selector
                            {
                                array.get_mut(i).and_then(|nested| {
                                    nested
                                        .as_object_mut()
                                        .and_then(|matched| matched.insert(nk.clone(), to.clone()))
                                });
                            } else {
                                array.insert(i, to.clone());
                            }
                            break;
                        }
                        array.get_mut(i).unwrap()
                    } else {
                        break;
                    }
                }
            };
        }
    });
    Ok(res)
}

pub enum MapAction<T> {
    ReplaceWith(T),
    Delete,
}

pub fn map_each(
    path: &str,
    val: &Value,
    fun: &mut dyn FnMut(&Value) -> MapAction<Value>,
) -> Result<Value, String> {
    let selectors = parser::parse(path)?;
    let paths = matches(selectors, val)?;

    let mut res = val.clone();
    let mut prefix = vec![];
    paths.iter().for_each(|path| {
        if !prefix.is_empty() && path.starts_with(&prefix) {
            return;
        }
        prefix = path.clone();
        let mut segments = path.iter().peekable();
        let mut cur = Some(&mut res);
        while let Some(segment) = segments.next() {
            if cur.is_none() {
                break;
            }
            cur = match segment {
                PathSegment::MemberName(k) => {
                    let object = cur.unwrap().as_object_mut().unwrap();
                    if segments.peek().is_none() {
                        match fun(object.get(&k.clone()).unwrap()) {
                            MapAction::Delete => {
                                object.remove(&k.clone());
                            }
                            MapAction::ReplaceWith(v) => {
                                object.insert(k.clone(), v);
                            }
                        }
                        break;
                    }
                    object.get_mut(&k.clone())
                }
                PathSegment::ArrayIndex(i) => {
                    let array = cur.unwrap().as_array_mut().unwrap();
                    if segments.peek().is_none() {
                        match fun(array.get(*i).unwrap()) {
                            MapAction::Delete => {
                                array.remove(*i);
                            }
                            MapAction::ReplaceWith(v) => {
                                array.insert(*i, v);
                            }
                        }
                        break;
                    }
                    array.get_mut(*i)
                }
            };
        }
    });
    Ok(res)
}

#[derive(Clone, Debug, PartialEq)]
pub enum PathSegment {
    MemberName(String),
    ArrayIndex(usize),
}

enum ConsList<T> {
    Cons(T, Rc<ConsList<T>>),
    Nil,
}

macro_rules! cons {
    ($list:expr,$elem:expr) => {
        Rc::new(ConsList::Cons($elem, Rc::clone(&$list)))
    };
}

// Protection against abusive selectors
const MAX_ACCUMULATOR_SIZE: usize = 1000;

pub fn matches(
    selectors: Vec<parser::Selector>,
    val: &Value,
) -> Result<Vec<Vec<PathSegment>>, String> {
    type ConsPath = Rc<ConsList<PathSegment>>;

    selectors
        .into_iter()
        .fold(Ok(vec![]), |acc, node| {
            let acc: Vec<(ConsPath, &Value)> = acc?;
            if acc.len() > MAX_ACCUMULATOR_SIZE {
                return Err(format!("too many passthrough matches ({})", acc.len()));
            };
            Ok(match node {
                parser::Selector::Root => vec![(Rc::new(ConsList::Nil), val)],
                parser::Selector::DotMemberName(k) => acc
                    .into_iter()
                    .filter_map(|(p, v)| {
                        v.as_object()
                            .and_then(|object| object.get(&k))
                            .map(|val| (cons!(p, PathSegment::MemberName(k.clone())), val))
                    })
                    .collect(),
                parser::Selector::ArrayIndex(i) => acc
                    .into_iter()
                    .filter_map(|(p, v)| {
                        v.as_array().and_then(|array| {
                            wrapped_index(i, array.len()).map(|safe_index| {
                                let elem = unsafe { array.get_unchecked(safe_index) };
                                (cons!(p, PathSegment::ArrayIndex(safe_index)), elem)
                            })
                        })
                    })
                    .collect(),
                parser::Selector::Wildcard => {
                    acc.into_iter()
                        .flat_map(|(p, v)| {
                            let mut col: Vec<(ConsPath, &Value)> = vec![];
                            if let Some(object) = v.as_object() {
                                col.extend(object.into_iter().map(|(key, val)| {
                                    (cons!(p, PathSegment::MemberName(key.clone())), val)
                                }));
                            }
                            if let Some(array) = v.as_array() {
                                col.extend(array.iter().enumerate().map(|(idx, val)| {
                                    (cons!(p, PathSegment::ArrayIndex(idx)), val)
                                }));
                            }
                            col
                        })
                        .collect()
                }
                parser::Selector::DecendantDotMemberName(k) => acc
                    .into_iter()
                    .flat_map(|(p, v)| {
                        let mut col: Vec<(ConsPath, &Value)> = vec![];
                        let mut cur: Vec<(ConsPath, &Value)> = vec![(p, v)];
                        while let Some((pp, next)) = cur.pop() {
                            if let Some(object) = next.as_object() {
                                cur.extend(object.into_iter().map(|(key, val)| {
                                    (cons!(pp, PathSegment::MemberName(key.clone())), val)
                                }));
                                if let Some(val) = object.get(&k.clone()) {
                                    let key = k.clone();
                                    col.push((cons!(pp, PathSegment::MemberName(key)), val));
                                };
                            }
                            if let Some(array) = next.as_array() {
                                cur.extend(array.iter().enumerate().map(|(idx, val)| {
                                    (cons!(pp, PathSegment::ArrayIndex(idx)), val)
                                }));
                            }
                        }
                        col
                    })
                    .collect(),
                parser::Selector::DecendantArrayIndex(i) => acc
                    .into_iter()
                    .flat_map(|(p, v)| {
                        let mut col: Vec<(ConsPath, &Value)> = vec![];
                        let mut cur: Vec<(ConsPath, &Value)> = vec![(p, v)];
                        while let Some((pp, next)) = cur.pop() {
                            if let Some(object) = next.as_object() {
                                cur.extend(object.into_iter().map(|(key, val)| {
                                    (cons!(pp, PathSegment::MemberName(key.clone())), val)
                                }));
                            }
                            if let Some(array) = next.as_array() {
                                cur.extend(array.iter().enumerate().map(|(idx, val)| {
                                    (cons!(pp, PathSegment::ArrayIndex(idx)), val)
                                }));
                                if let Some(safe_index) = wrapped_index(i, array.len()) {
                                    let elem = unsafe { array.get_unchecked(safe_index) };
                                    col.push((
                                        cons!(pp, PathSegment::ArrayIndex(safe_index)),
                                        elem,
                                    ));
                                }
                            }
                        }
                        col
                    })
                    .collect(),
                parser::Selector::DecendantWildcard => acc
                    .into_iter()
                    .flat_map(|(p, v)| {
                        let mut col: Vec<(ConsPath, &Value)> = vec![];
                        let mut cur: Vec<(ConsPath, &Value)> = vec![(p, v)];
                        while let Some((pp, next)) = cur.pop() {
                            if let Some(object) = next.as_object() {
                                cur.extend(object.into_iter().map(|(key, val)| {
                                    (cons!(pp, PathSegment::MemberName(key.clone())), val)
                                }));
                                col.extend(object.into_iter().map(|(key, val)| {
                                    (cons!(pp, PathSegment::MemberName(key.clone())), val)
                                }));
                            }
                            if let Some(array) = next.as_array() {
                                cur.extend(array.iter().enumerate().map(|(idx, val)| {
                                    (cons!(pp, PathSegment::ArrayIndex(idx)), val)
                                }));
                                col.extend(array.iter().enumerate().map(|(idx, val)| {
                                    (cons!(pp, PathSegment::ArrayIndex(idx)), val)
                                }));
                            }
                        }
                        col
                    })
                    .collect(),
                parser::Selector::Union(union_elements) => acc
                    .into_iter()
                    .flat_map(|(p, v)| {
                        union_elements
                            .clone()
                            .into_iter()
                            .filter_map(move |union_element| match union_element {
                                parser::UnionMember::MemberName(k) => v
                                    .as_object()
                                    .and_then(|object| object.get(&k))
                                    .map(|val| (cons!(p, PathSegment::MemberName(k)), val)),
                                parser::UnionMember::ArrayIndex(i) => {
                                    v.as_array().and_then(|array| {
                                        wrapped_index(i, array.len()).map(|safe_index| {
                                            let elem = unsafe { array.get_unchecked(safe_index) };
                                            (cons!(p, PathSegment::ArrayIndex(safe_index)), elem)
                                        })
                                    })
                                }
                            })
                    })
                    .collect(),
                parser::Selector::ArraySlice(start, end, step) => acc
                    .into_iter()
                    .flat_map(|(p, v)| {
                        let mut col: Vec<(ConsPath, &Value)> = vec![];
                        if let Some(array) = v.as_array() {
                            let step = step.unwrap_or(1);
                            let len = array.len() as isize;
                            let start =
                                start.unwrap_or(if step >= 0 { 0_isize } else { len - 1_isize });
                            let end = end.unwrap_or({
                                if step >= 0 {
                                    array.len() as isize
                                } else {
                                    -len - 1
                                }
                            });
                            let (lower, upper) = array_bounds(start, end, step, len);
                            let mut i;
                            if step > 0 {
                                i = lower;
                                while i < upper {
                                    col.push((
                                        cons!(p, PathSegment::ArrayIndex(i as usize)),
                                        array.get(i as usize).unwrap(),
                                    ));
                                    i += step;
                                }
                            }
                            if step < 0 {
                                i = upper;
                                while lower < i {
                                    col.push((
                                        cons!(p, PathSegment::ArrayIndex(i as usize)),
                                        array.get(i as usize).unwrap(),
                                    ));
                                    i += step;
                                }
                            }
                        }
                        col
                    })
                    .collect(),
            })
        })
        .map(|paths| {
            paths
                .into_iter()
                .map(|(cons_path, _)| {
                    // TODO: this is a performance bottleneck for large result sets, find a better datastructure
                    let mut cur = &cons_path;
                    let mut res = vec![];
                    while let ConsList::Cons(head, tail) = cur.as_ref() {
                        cur = tail;
                        res.push(head.clone());
                    }
                    res.into_iter().rev().collect()
                })
                .collect()
        })
}

fn normalize_slice_bound(i: isize, len: isize) -> isize {
    if i >= 0 {
        i
    } else {
        len + i
    }
}

fn array_bounds(start: isize, end: isize, step: isize, len: isize) -> (isize, isize) {
    let n_start = normalize_slice_bound(start, len);
    let n_end = normalize_slice_bound(end, len);

    if step >= 0 {
        (n_start.clamp(0, len), n_end.clamp(0, len))
    } else {
        // spec allow retuning -1, but after normalize , -ve values feel wrong
        (n_end.clamp(-1, len - 1), n_start.clamp(-1, len - 1))
        // (n_end.clamp(0, len - 1), n_start.clamp(0, len - 1))
    }
}

fn wrapped_index(idx: isize, len: usize) -> Option<usize> {
    if idx < 0 && idx.unsigned_abs() > len {
        return None;
    }
    if idx >= 0 {
        Some(idx as usize)
    } else {
        let positive_idx = idx.checked_add_unsigned(len);
        positive_idx.map(|i| i as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::json;

    #[test]
    fn get_success_tests() {
        struct Expectation<'a> {
            path: &'a str,
            expect: Vec<&'a Value>,
        }
        struct Test<'a> {
            input: Value,
            expectations: Vec<Expectation<'a>>,
        }
        vec![
            Test {
                input: json!({
                    "a": "foo",
                    "b": "bar",
                    "c": [
                        "1",
                        "2",
                        "3"
                    ]
                }),
                expectations: vec![
                    Expectation {
                        path: "$.c",
                        expect: vec![&json!(["1", "2", "3"])],
                    },
                    Expectation {
                        path: "$['a']",
                        expect: vec![&json!("foo")],
                    },
                    Expectation {
                        path: "$.c[0]",
                        expect: vec![&json!("1")],
                    },
                    Expectation {
                        path: "$.c[1]",
                        expect: vec![&json!("2")],
                    },
                    Expectation {
                        path: "$.c.*",
                        expect: vec![&json!("1"), &json!("2"), &json!("3")],
                    },
                    Expectation {
                        path: "$['a','b']",
                        expect: vec![&json!("foo"), &json!("bar")],
                    },
                    Expectation {
                        path: "$['a','d']",
                        expect: vec![&json!("foo")],
                    },
                    Expectation {
                        path: "$.c[0,-1]",
                        expect: vec![&json!("1"), &json!("3")],
                    },
                    Expectation {
                        path: "$.c[0,-4]",
                        expect: vec![&json!("1")],
                    },
                    Expectation {
                        path: "$.c[0:1]",
                        expect: vec![&json!("1")],
                    },
                    Expectation {
                        path: "$.c[0:1:1]",
                        expect: vec![&json!("1")],
                    },
                    Expectation {
                        path: "$.c[0:1:0]",
                        expect: vec![],
                    },
                    Expectation {
                        path: "$.c[0:1:-1]",
                        expect: vec![],
                    },
                    Expectation {
                        path: "$.c[1:0:-1]",
                        expect: vec![&json!("2")],
                    },
                    Expectation {
                        path: "$.c[-1:1:-1]",
                        expect: vec![&json!("3")],
                    },
                    Expectation {
                        path: "$.c[-1:0:-1]",
                        expect: vec![&json!("3"), &json!("2")],
                    },
                    Expectation {
                        path: "$.c[-1:0:-1]",
                        expect: vec![&json!("3"), &json!("2")],
                    },
                    Expectation {
                        path: "$.c[-100:100:-1]",
                        expect: vec![],
                    },
                    Expectation {
                        path: "$.c[-100:100]",
                        expect: vec![&json!("1"), &json!("2"), &json!("3")],
                    },
                ],
            },
            Test {
                input: json!({
                    "a": {"b": "c"},
                    "d": "e",
                    "f": {"b": "g"},
                    "h": {"j": {"b":"k"}}
                }),
                expectations: vec![
                    Expectation {
                        path: "$.*.b",
                        expect: vec![&json!("c"), &json!("g")],
                    },
                    Expectation {
                        path: "$['h'].*.b",
                        expect: vec![&json!("k")],
                    },
                ],
            },
            Test {
                input: json!(["a", "b", "c"]),
                expectations: vec![
                    Expectation {
                        path: "$.b",
                        expect: vec![],
                    },
                    Expectation {
                        path: "$.*",
                        expect: vec![&json!("a"), &json!("b"), &json!("c")],
                    },
                    Expectation {
                        path: "$[-1]",
                        expect: vec![&json!("c")],
                    },
                    Expectation {
                        path: "$[-2]",
                        expect: vec![&json!("b")],
                    },
                    Expectation {
                        path: "$[-3]",
                        expect: vec![&json!("a")],
                    },
                    Expectation {
                        path: "$[0]",
                        expect: vec![&json!("a")],
                    },
                    Expectation {
                        path: "$[1]",
                        expect: vec![&json!("b")],
                    },
                    Expectation {
                        path: "$[2]",
                        expect: vec![&json!("c")],
                    },
                    Expectation {
                        path: "$[-5]",
                        expect: vec![],
                    },
                ],
            },
            Test {
                input: json!({
                    "a": {
                        "x": "1"
                    },
                    "c": {
                        "x": "2"
                    }
                }),
                expectations: vec![Expectation {
                    path: "$..x",
                    expect: vec![&json!("2"), &json!("1")],
                }],
            },
            Test {
                input: json!({
                    "a": {
                        "x": "1"
                    },
                    "c": {
                        "x": {
                            "x": "2"
                        }
                    }
                }),
                expectations: vec![
                    Expectation {
                        path: "$..x.x",
                        expect: vec![&json!("2")],
                    },
                    Expectation {
                        path: "$..*.x",
                        expect: vec![&json!("1"), &json!({"x": "2"}), &json!("2")],
                    },
                    Expectation {
                        path: "$..x",
                        expect: vec![&json!({"x": "2"}), &json!("2"), &json!("1")],
                    },
                ],
            },
        ]
        .iter()
        .for_each(|test| {
            test.expectations
                .iter()
                .for_each(|e| assert_eq!(get(e.path, &test.input).expect("error get"), e.expect));
        });
    }

    #[test]
    fn fuzzing_crash_on_explosion_in_passthrough_matches() {
        fn array(level: usize) -> Value {
            if level == 0 {
                return Value::Array(vec![]);
            }
            Value::Array(vec![array(level - 1)])
        }

        struct Test<'a> {
            input: Value,
            path: &'a str,
        }
        vec![Test {
            input: array(35),
            path: "$..*..*..*..*..*..*",
        }]
        .iter()
        .for_each(|test| {
            let selectors = parser::parse(test.path).expect("error parse");
            let _ = matches(selectors, &test.input);
        });
    }

    #[test]
    fn del_success_tests() {
        struct Expectation<'a> {
            path: &'a str,
            expect: Value,
        }
        struct Test<'a> {
            input: Value,
            expectations: Vec<Expectation<'a>>,
        }
        vec![
            Test {
                input: json!({
                    "a": 1,
                    "b": 2,
                }),
                expectations: vec![Expectation {
                    path: "$.a",
                    expect: json!({"b": 2}),
                }],
            },
            Test {
                input: json!([1, 2, 3]),
                expectations: vec![
                    Expectation {
                        path: "$[1]",
                        expect: json!([1, 3]),
                    },
                    Expectation {
                        path: "$[-1]",
                        expect: json!([1, 2]),
                    },
                ],
            },
            Test {
                input: json!({
                    "a": {
                        "x": "1"
                    },
                    "c": {
                        "x": {
                            "x": "2"
                        }
                    }
                }),
                expectations: vec![
                    Expectation {
                        path: "$..x.x",
                        expect: json!({
                        "a": {
                            "x": "1"
                        },
                        "c": {
                            "x": {}
                        }
                        }),
                    },
                    Expectation {
                        path: "$..x",
                        expect: json!({
                        "a": {},
                        "c": {}
                        }),
                    },
                    Expectation {
                        path: "$..*",
                        expect: json!({}),
                    },
                ],
            },
            Test {
                input: json!({
                    "a": 1,
                    "b": {"a": 2, "c": 3}
                }),
                expectations: vec![Expectation {
                    path: "$..a",
                    expect: json!({
                        "b": {"c": 3}
                    }),
                }],
            },
        ]
        .iter()
        .for_each(|test| {
            test.expectations.iter().for_each(|e| {
                assert_eq!(
                    map_each(e.path, &test.input, &mut |_| MapAction::Delete).expect("error del"),
                    e.expect
                )
            });
        });
    }

    #[test]
    fn set_success_tests() {
        struct Expectation<'a> {
            path: &'a str,
            set_to: Value,
            expect: Value,
        }
        struct Test<'a> {
            input: Value,
            expectations: Vec<Expectation<'a>>,
        }
        vec![
            Test {
                input: json!({
                    "a": 1,
                    "b": {"a": 2, "c": 3}
                }),
                expectations: vec![
                    Expectation {
                        path: "$..a",
                        set_to: json!("foo"),
                        expect: json!({
                            "a": "foo",
                            "b": {"a": "foo", "c": 3}
                        }),
                    },
                    Expectation {
                        path: "$..*",
                        set_to: json!(1),
                        expect: json!({
                            "a": 1,
                            "b": 1,
                        }),
                    },
                    Expectation {
                        path: "$",
                        set_to: json!(1),
                        expect: json!(1),
                    },
                    Expectation {
                        path: "..*",
                        set_to: json!({"a": "c"}),
                        expect: json!({
                            "a": {"a": "c"},
                            "b": {"a": "c"},
                        }),
                    },
                ],
            },
            Test {
                input: json!({
                    "a": {"b":{"c": 1}},
                    "d": {"b":{"c": 2}},
                }),
                expectations: vec![Expectation {
                    path: "$..b.d",
                    set_to: json!(3),
                    expect: json!({
                        "a": {"b":{"c": 1, "d": 3}},
                        "d": {"b":{"c": 2, "d": 3}},
                    }),
                }],
            },
            Test {
                input: json!({
                    "a": 1
                }),
                expectations: vec![Expectation {
                    path: "$.b",
                    set_to: json!(2),
                    expect: json!({
                        "a": 1,
                        "b": 2,
                    }),
                }],
            },
            Test {
                input: json!({
                    "a": 1, "b": 2
                }),
                expectations: vec![Expectation {
                    path: "$.a",
                    set_to: json!(2),
                    expect: json!({
                        "a": 2,
                        "b": 2,
                    }),
                }],
            },
        ]
        .iter()
        .for_each(|test| {
            test.expectations.iter().for_each(|e| {
                assert_eq!(
                    set(e.path, &test.input, &e.set_to).expect("error set"),
                    e.expect
                )
            });
        });
    }
}
