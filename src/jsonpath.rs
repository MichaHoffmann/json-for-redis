use jsonpath_rs::parser::{parse, Selector};
use jsonpath_rs::{matches, PathSegment};
use redis_module::RedisError;
use serde_json::Value;

pub fn get<'a>(path: &str, val: &'a Value) -> Result<Vec<&'a Value>, RedisError> {
    let selectors = match parse(path) {
        Ok(v) => v,
        Err(e) => return Err(RedisError::String(e)),
    };

    if selectors == vec![Selector::Root] {
        return Ok(vec![val]);
    }
    let paths = match matches(selectors, val) {
        Ok(v) => v,
        Err(e) => return Err(RedisError::String(e)),
    };

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

pub fn set(path: &str, val: &Value, to: &Value) -> Result<Value, RedisError> {
    let mut selectors = match parse(path) {
        Ok(v) => v,
        Err(e) => return Err(RedisError::String(e)),
    };

    if selectors == vec![Selector::Root] {
        return Ok(to.clone());
    }

    // When the last element is a DotMemberName selector we also set it. To do this
    // we need to match to the second to last element and if we did so and this
    // element is an object we add/update that key.
    let maybe_last_selector = match selectors.last().unwrap() {
        Selector::DotMemberName(_) => selectors.pop(),
        _ => None,
    };

    let mut res = val.clone();
    if selectors == vec![Selector::Root] {
        if let Some(Selector::DotMemberName(k)) = &maybe_last_selector {
            res.as_object_mut()
                .and_then(|obj| obj.insert(k.clone(), to.clone()));
        }
        return Ok(res);
    }

    let paths = match matches(selectors, val) {
        Ok(v) => v,
        Err(e) => return Err(RedisError::String(e)),
    };

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
                            if let Some(Selector::DotMemberName(nk)) = &maybe_last_selector {
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
                            if let Some(Selector::DotMemberName(nk)) = &maybe_last_selector {
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
) -> Result<Value, RedisError> {
    let selectors = match parse(path) {
        Ok(v) => v,
        Err(e) => return Err(RedisError::String(e)),
    };

    let paths = match matches(selectors, val) {
        Ok(v) => v,
        Err(e) => return Err(RedisError::String(e)),
    };

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
