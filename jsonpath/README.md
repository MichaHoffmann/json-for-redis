# jsonpath-rs

An implementation of JSONPath according to https://www.ietf.org/archive/id/draft-ietf-jsonpath-base-03.html

## Features

- [X] Root Selector `$`
- [X] Dot Selector `.foo`
- [X] Dot Wildcard Selector `.*`
- [X] Index Selector `["foo"]`, `[2]`
- [X] Index Wildcard Selector `[*]`
- [ ] Array Slice Selector `[0:6:2]`
- [X] Decendent Selector `..foo`, `..[2]`, `..*`, `..[*]`
- [X] Union Selector `["foo", "bar"]`
- [ ] Filter Selector `[?@]`
