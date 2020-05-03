#[cfg(test)]
mod tests {
    //Referring to https://package.elm-lang.org/packages/elm/parser/latest/Parser

    use std::cmp::PartialEq;
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
    struct TestparserElement {
        el_type: Option<TestparserElementType>,
        i64: Option<i64>,
    }

    impl TestparserElement {
        fn new() -> TestparserElement {
            TestparserElement {
                el_type: None,
                i64: None,
            }
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    enum TestparserElementType {
        Int64,
    }
    #[derive(Debug, Clone)]
    struct Testparser {
        input_original: String,
        input_remaining: String,
        output: Vec<TestparserElement>,
        chomp: String,
        success: bool,
    }

    impl Testparser {
        fn new(input_string: &str) -> Testparser {
            Testparser {
                input_original: input_string.to_string(),
                input_remaining: input_string.to_string(),
                chomp: "".to_string(),
                output: Vec::<TestparserElement>::new(),
                success: true,
            }
        }

        fn word(mut self: Testparser, expected: &str) -> Testparser {
            match self.clone().input_remaining.get(0..expected.len()) {
                Some(next) if next == expected => {
                    self.input_remaining = self.input_remaining[expected.len()..].to_string();
                    self.chomp += next;
                    self.success = true;
                    self
                }
                _ => {
                    self.success = false;
                    self
                }
            }
        }

        fn char(mut self: Testparser) -> Testparser {
            match self.input_remaining.chars().next() {
                Some(next) => {
                    self.input_remaining = self.input_remaining[next.len_utf8()..].to_string();
                    self.chomp += next.encode_utf8(&mut [0; 1]);
                    self.success = true;
                    self
                }
                _ => {
                    self.success = false;
                    self
                }
            }
        }

        fn digit(mut self: Testparser) -> Testparser {
            match self.input_remaining.chars().next() {
                Some(next) if next.is_digit(10) => {
                    self.input_remaining = self.input_remaining[next.len_utf8()..].to_string();
                    self.chomp += next.encode_utf8(&mut [0; 1]);
                    self.success = true;
                    self
                }
                _ => {
                    self.success = false;
                    self
                }
            }
        }

        fn one_or_more_of<F>(mut self: Testparser, func: F) -> Testparser
        where
            F: Fn(Testparser) -> Testparser,
        {
            let chomp = self.clone().chomp;
            while self.success {
                self = func(self)
            }
            if self.chomp == chomp {
                self.success = false;
                self
            } else {
                self.success = true;
                self
            }
        }

        fn zero_or_more_of<F>(mut self: Testparser, func: F) -> Testparser
        //always succeeds
        where
            F: Fn(Testparser) -> Testparser,
        {
            while self.success {
                self = func(self)
            }
            self.success = true;
            self
        }

        fn optional<F>(mut self: Testparser, func: F) -> Testparser
        //always succeeds
        where
            F: Fn(Testparser) -> Testparser,
        {
            self = func(self);
            self.success = true;
            self
        }

        fn int(mut self: Testparser) -> Testparser {
            self = self
                .optional(|s: Testparser| Testparser::word(s, "-"))
                .one_or_more_of(Testparser::digit);

            if self.success {
                let mut el = TestparserElement::new();
                let val = self.clone().chomp.parse().unwrap();
                el.el_type = Some(TestparserElementType::Int64);
                el.i64 = Some(val);
                self.output.push(el);
                self
            } else {
                self
            }
        }
    }

    #[test]
    fn test_int() {
        //not an int
        let mut testparser = Testparser::new("a123");
        let result = testparser.clone().int();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "a123");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, false);

        //positive small int
        testparser = Testparser::new("12");
        let result = testparser.clone().int();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(TestparserElementType::Int64));
        assert_eq!(result.output[0].i64, Some(12));
        assert_eq!(result.chomp, "12");
        assert_eq!(result.success, true);

        //positive large int
        testparser = Testparser::new("123456");
        let result = testparser.clone().int();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.chomp, "123456");
        assert_eq!(result.success, true);

        //negative int
        testparser = Testparser::new("-123456");
        let result = testparser.clone().int();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.chomp, "-123456");
        assert_eq!(result.success, true);
    }

    #[test]
    fn test_optional() {
        let mut testparser = Testparser::new("a123Test");
        let result = testparser.clone().optional(Testparser::char);
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "123Test");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "a");
        assert_eq!(result.success, true);

        testparser = Testparser::new("a123Test");
        let result = testparser.clone().zero_or_more_of(Testparser::digit);
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "a123Test");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);
    }

    #[test]
    fn test_zero_or_more_of() {
        let mut testparser = Testparser::new("a123Test");
        let result = testparser.clone().zero_or_more_of(Testparser::digit);
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "a123Test");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        testparser = Testparser::new("123Test");
        let result = testparser.clone().zero_or_more_of(Testparser::digit);
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "Test");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "123");
        assert_eq!(result.success, true);
    }
    #[test]
    fn test_one_or_more_of() {
        let mut testparser = Testparser::new("a123Test");
        let result = testparser.clone().one_or_more_of(Testparser::digit);
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "a123Test");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, false);

        testparser = Testparser::new("123Test");
        let result = testparser.clone().one_or_more_of(Testparser::digit);
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "Test");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "123");
        assert_eq!(result.success, true);
    }

    #[test]
    fn test_multiple_parsers() {
        let testparser = Testparser::new("1Test");
        let result = testparser.clone().digit().word("Te");
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "st");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "1Te");
        assert_eq!(result.success, true);
    }
    #[test]
    fn test_digit() {
        let testparser = Testparser::new("123Test");
        let result = testparser.clone().digit().digit().digit();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "Test");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "123");
        assert_eq!(result.success, true);
    }
    #[test]
    fn test_char() {
        let testparser = Testparser::new("Testing 123");
        let result = testparser.clone().char().char().char().char();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "ing 123");
        assert_eq!(result.output.len(), 0);
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
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "Testing 123");
        assert_eq!(result.success, true);
    }
}
