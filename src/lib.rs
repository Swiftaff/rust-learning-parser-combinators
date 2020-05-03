#[cfg(test)]
mod tests {
    //Referring to https://package.elm-lang.org/packages/elm/parser/latest/Parser

    /*
    map?
    pred?
    and_then?
    parse
    the_letter_a
    match_literal
    identifier
    pair
    left
    right
    one_or_more
    zero_or_more
    any_char
    whitespace_char
    space_one_or_more
    space_zero_or_more
    quoted_string
    attribute_pair
    attributes
    element_start
    single_element
    open_element
    either
    element
    close_element
    parent_element
    whitespace_wrap
    */

    #[derive(Debug, Clone)]
    struct Testparser {
        input_original: String,
        input_remaining: String,
        output: String,
        chomp: String,
        success: bool,
    }

    impl Testparser {
        fn new(input_string: &str) -> Testparser {
            Testparser {
                input_original: input_string.to_string(),
                input_remaining: input_string.to_string(),
                chomp: "".to_string(),
                output: "".to_string(),
                success: true,
            }
        }

        fn word(self: Testparser, expected: &str) -> Testparser {
            match self.input_remaining.get(0..expected.len()) {
                Some(next) if next == expected => {
                    let mut new_parser = self.clone();
                    new_parser.input_remaining =
                        new_parser.input_remaining[expected.len()..].to_string();
                    new_parser.output += "word";
                    new_parser.chomp += next;
                    new_parser.success = true;
                    new_parser
                }
                _ => {
                    let mut new_parser = self;
                    new_parser.success = false;
                    new_parser
                }
            }
        }

        fn char(self: Testparser) -> Testparser {
            match self.input_remaining.chars().next() {
                Some(next) => {
                    let mut new_parser = self;
                    new_parser.input_remaining =
                        new_parser.input_remaining[next.len_utf8()..].to_string();
                    new_parser.output += "char";
                    new_parser.chomp += next.encode_utf8(&mut [0; 1]);
                    new_parser.success = true;
                    new_parser
                }
                _ => {
                    let mut new_parser = self;
                    new_parser.success = false;
                    new_parser
                }
            }
        }

        fn digit(self: Testparser) -> Testparser {
            match self.input_remaining.chars().next() {
                Some(next) if next.is_digit(10) => {
                    let mut new_parser = self;
                    new_parser.input_remaining =
                        new_parser.input_remaining[next.len_utf8()..].to_string();
                    new_parser.output += "digit";
                    new_parser.chomp += next.encode_utf8(&mut [0; 1]);
                    new_parser.success = true;
                    new_parser
                }
                _ => {
                    let mut new_parser = self;
                    new_parser.success = false;
                    new_parser
                }
            }
        }

        fn one_or_more_of<F>(self: Testparser, func: F) -> Testparser
        where
            F: Fn(Testparser) -> Testparser,
        {
            let mut new_parser = self;
            let chomp = new_parser.clone().chomp;
            while new_parser.success {
                new_parser = func(new_parser)
            }
            if new_parser.chomp == chomp {
                new_parser.success = false;
                new_parser
            } else {
                new_parser.success = true;
                new_parser
            }
        }

        fn zero_or_more_of<F>(self: Testparser, func: F) -> Testparser
        //always succeeds
        where
            F: Fn(Testparser) -> Testparser,
        {
            let mut new_parser = self;
            while new_parser.success {
                new_parser = func(new_parser)
            }
            new_parser.success = true;
            new_parser
        }

        fn optional<F>(self: Testparser, func: F) -> Testparser
        //always succeeds
        where
            F: Fn(Testparser) -> Testparser,
        {
            let mut new_parser = self;
            new_parser = func(new_parser);
            new_parser.success = true;
            new_parser
        }

        fn int(self: Testparser) -> Testparser {
            self.optional(|s: Testparser| Testparser::word(s, "-"))
                .one_or_more_of(Testparser::digit)
        }
    }

    //type TestparserFunction<S> = Rc<Fn(S) -> S>;

    //enum ParserName {
    //    Digit,
    //    Char,
    //}

    #[test]
    fn test_int() {
        //not an int
        let mut testparser = Testparser::new("a123");
        let result = testparser.clone().int();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "a123");
        assert_eq!(result.output, "");
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, false);

        //positive small int
        testparser = Testparser::new("12");
        let result = testparser.clone().int();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output, "digitdigit");
        assert_eq!(result.chomp, "12");
        assert_eq!(result.success, true);

        //positive large int
        testparser = Testparser::new("123456");
        let result = testparser.clone().int();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output, "digitdigitdigitdigitdigitdigit");
        assert_eq!(result.chomp, "123456");
        assert_eq!(result.success, true);

        //negative int
        testparser = Testparser::new("-123456");
        let result = testparser.clone().int();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output, "worddigitdigitdigitdigitdigitdigit");
        assert_eq!(result.chomp, "-123456");
        assert_eq!(result.success, true);
    }

    #[test]
    fn test_optional() {
        let mut testparser = Testparser::new("a123Test");
        let result = testparser.clone().optional(Testparser::char);
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "123Test");
        assert_eq!(result.output, "char");
        assert_eq!(result.chomp, "a");
        assert_eq!(result.success, true);

        testparser = Testparser::new("a123Test");
        let result = testparser.clone().zero_or_more_of(Testparser::digit);
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "a123Test");
        assert_eq!(result.output, "");
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);
    }

    #[test]
    fn test_zero_or_more_of() {
        let mut testparser = Testparser::new("a123Test");
        let result = testparser.clone().zero_or_more_of(Testparser::digit);
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "a123Test");
        assert_eq!(result.output, "");
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        testparser = Testparser::new("123Test");
        let result = testparser.clone().zero_or_more_of(Testparser::digit);
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "Test");
        assert_eq!(result.output, "digitdigitdigit");
        assert_eq!(result.chomp, "123");
        assert_eq!(result.success, true);
    }
    #[test]
    fn test_one_or_more_of() {
        let mut testparser = Testparser::new("a123Test");
        let result = testparser.clone().one_or_more_of(Testparser::digit);
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "a123Test");
        assert_eq!(result.output, "");
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, false);

        testparser = Testparser::new("123Test");
        let result = testparser.clone().one_or_more_of(Testparser::digit);
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "Test");
        assert_eq!(result.output, "digitdigitdigit");
        assert_eq!(result.chomp, "123");
        assert_eq!(result.success, true);
    }

    #[test]
    fn test_multiple_parsers() {
        let testparser = Testparser::new("1Test");
        let result = testparser.clone().digit().word("Te");
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "st");
        assert_eq!(result.output, "digitword");
        assert_eq!(result.chomp, "1Te");
        assert_eq!(result.success, true);
    }
    #[test]
    fn test_digit() {
        let testparser = Testparser::new("123Test");
        let result = testparser.clone().digit().digit().digit();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "Test");
        assert_eq!(result.output, "digitdigitdigit");
        assert_eq!(result.chomp, "123");
        assert_eq!(result.success, true);
    }
    #[test]
    fn test_char() {
        let testparser = Testparser::new("Testing 123");
        let result = testparser.clone().char().char().char().char();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "ing 123");
        assert_eq!(result.output, "charcharcharchar");
        assert_eq!(result.chomp, "Test");
        assert_eq!(result.success, true);
    }

    #[test]
    fn test_word() {
        let testparser = Testparser::new("Testing 123");
        let result = testparser
            .clone()
            .word("Test")
            .word("ing")
            .word(" ")
            .word("123");
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output, "wordwordwordword");
        assert_eq!(result.chomp, "Testing 123");
        assert_eq!(result.success, true);
    }
}
