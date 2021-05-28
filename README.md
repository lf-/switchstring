# switchstring

it's a string, but unlike a string where you can add substrings, you can
subtract substrings here as well. it is thus a switchstring

## why

to make mathematicians (and maybe you) suffer of the knowledge that this exists

## how

```rust
use switchstring::Switchstring;

let a = "I promise I love maths";
let b = "maths";
let c = "cute rustaceans such as ferris";
let d: Switchstring = "I promise ".into();
let improved = String::from(-d + a - b + c);
assert_eq!("I love cute rustaceans such as ferris", improved);
```
