#[derive(Clone, Debug, PartialEq, Eq)]
struct Element {
    name: String,
    attributes: Vec<(String, String)>,
    children: Vec<Element>,
}

fn the_letter_a(input: &str) -> Result<(&str, ()), &str> {
    match input.chars().next() {
        Some('a') => Ok((&input['a'.len_utf8()..], ())),
        _ => Err(input),
    }
}

fn match_literal(expected: &'static str) -> impl Fn(&str) -> Result<(&str, ()), &str> {
    move |input| match input.get(0..expected.len()) {
        Some(next) if next == expected => Ok((&input[expected.len()..], ())),
        _ => Err(input),
    }
}

#[test]
fn test_literal_parser() {
    let parse_joe = match_literal("Hello Joe!");
    assert_eq!(parse_joe("Hello Joe!"), Ok(("", ())));
    assert_eq!(
        parse_joe("Hello Joe! Hello Robert!"),
        Ok((" Hello Robert!", ()))
    );
    assert_eq!(parse_joe("Hello Mike!"), Err("Hello Mike!"));
}
