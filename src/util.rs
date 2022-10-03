pub fn to_quoted(s: &String) -> String {
    let mut r = s.replace("\"", "\\\"");
    r.insert_str(0, "\"");
    r.insert_str(r.len(), "\"");
    return r;
}
