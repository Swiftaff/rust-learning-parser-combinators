#[cfg(test)]
mod tests {
    //Referring to https://package.elm-lang.org/packages/elm/parser/latest/Parser

    //int
    //float
    //variable
    //fn variable_assign (=)
    //fn variable_sum (+)

    //= x + 1 2
    //print x
    //= y + 1.1 2.2
    //print y
    use std::cmp::PartialEq;
    use unicode_segmentation::UnicodeSegmentation;
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
        float64: Option<f64>,
        variable: Option<String>,
    }

    impl TestparserElement {
        fn new() -> TestparserElement {
            TestparserElement {
                el_type: None,
                i64: None,
                float64: None,
                variable: None,
            }
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    enum TestparserElementType {
        Int64,
        Float64,
        Variable,
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

        //PRIMITIVES

        fn word(mut self: Testparser, expected: &str) -> Testparser {
            //any series of characters, in the "expected" string
            if self.success {
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
            } else {
                self
            }
        }

        fn char(mut self: Testparser) -> Testparser {
            //a character, excluding ' '(space)
            if self.success {
                match self.clone().input_remaining.graphemes(true).next() {
                    Some(next) => {
                        if next == " " {
                            self.success = false;
                            self
                        } else {
                            self.input_remaining = self.input_remaining[next.len()..].to_string();
                            self.chomp += next;
                            self.success = true;
                            self
                        }
                    }
                    _ => {
                        self.success = false;
                        self
                    }
                }
            } else {
                self
            }
        }

        fn digit(mut self: Testparser) -> Testparser {
            //a single digit 0,1,2,3,4,5,6,7,8,9
            if self.success {
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
            } else {
                self
            }
        }

        //COMBINATORS

        fn one_or_more_of<F>(mut self: Testparser, func: F) -> Testparser
        //either one or multiple of any parser
        where
            F: Fn(Testparser) -> Testparser,
        {
            if self.success {
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
            } else {
                self
            }
        }

        fn zero_or_more_of<F>(mut self: Testparser, func: F) -> Testparser
        //always succeeds
        //either zero, one or multiple of any parser
        where
            F: Fn(Testparser) -> Testparser,
        {
            if self.success {
                while self.success {
                    self = func(self)
                }
                self.success = true;
                self
            } else {
                self
            }
        }

        fn optional<F>(mut self: Testparser, func: F) -> Testparser
        //always succeeds
        //either 1 or zero of any parser
        where
            F: Fn(Testparser) -> Testparser,
        {
            if self.success {
                self = func(self);
                self.success = true;
                self
            } else {
                self
            }
        }

        fn first_success_of<F>(mut self: Testparser, funcs: Vec<F>) -> Testparser
        where
            F: Fn(Testparser) -> Testparser,
        {
            if self.success {
                for func in funcs {
                    let new_self = func(self.clone());
                    if new_self.success {
                        return new_self;
                    }
                }
                self.success = false;
                return self;
            } else {
                return self;
            };
        }

        //HELPERS
        fn chomp_clear(mut self: Testparser) -> Testparser {
            self.chomp = "".to_string();
            self
        }

        //ELEMENTS/VALUES

        fn int(mut self: Testparser) -> Testparser {
            //integer number, e.g. 12 or -123456
            if self.success {
                self = self
                    .optional(|s: Testparser| Testparser::word(s, "-"))
                    .one_or_more_of(Testparser::digit);

                if self.success {
                    let mut el = TestparserElement::new();
                    let val = self.clone().chomp.parse().unwrap();
                    el.el_type = Some(TestparserElementType::Int64);
                    el.i64 = Some(val);
                    self.output.push(el);
                    self.chomp = "".to_string();
                    self
                } else {
                    self
                }
            } else {
                self
            }
        }

        fn float(mut self: Testparser) -> Testparser {
            //floating point number, e.g. 12.34 or -123.45
            if self.success {
                self = self
                    .optional(|s: Testparser| Testparser::word(s, "-"))
                    .one_or_more_of(Testparser::digit)
                    .word(".")
                    .one_or_more_of(Testparser::digit);

                if self.success {
                    let mut el = TestparserElement::new();
                    let val = self.clone().chomp.parse().unwrap();
                    el.el_type = Some(TestparserElementType::Float64);
                    el.float64 = Some(val);
                    self.output.push(el);
                    self.chomp = "".to_string();
                    self
                } else {
                    self
                }
            } else {
                self
            }
        }

        fn variable(mut self: Testparser) -> Testparser {
            //variable name of chars followed by a space, e.g. "x" or "lö̲ng_variablé_name"
            self = self.one_or_more_of(Testparser::char).word(" ");
            if self.success {
                let mut el = TestparserElement::new();
                let chomp = self.clone().chomp;
                let variable = chomp[..(chomp.len() - 1)].to_string();
                el.el_type = Some(TestparserElementType::Variable);
                el.variable = Some(variable);
                self.output.push(el);
                self.chomp = "".to_string();
                self
            } else {
                self
            }
        }

        //FUNCTIONS

        fn variable_assign(mut self: Testparser) -> Testparser {
            //equals sign, variable name, value (test using int for now), e.g. "= x 1" (x equals 1)
            self = self.word("= ").chomp_clear().variable().first_success_of(
                [Testparser::variable_sum, Testparser::float, Testparser::int].to_vec(),
            ); //float first so the number before . is not thought of as an int
            if self.success {
                let mut el = TestparserElement::new();
                let variable_el = self.output[self.output.len() - 2].clone();
                let value_el = self.output[self.output.len() - 1].clone();
                match value_el.el_type {
                    Some(TestparserElementType::Int64) => el.i64 = value_el.i64,
                    _ => el.float64 = value_el.float64,
                }
                el.el_type = Some(TestparserElementType::Variable);
                el.variable = variable_el.variable;
                self.output.remove(self.output.len() - 1);
                self.output.remove(self.output.len() - 1);
                self.output.push(el);
                self.chomp = "".to_string();
                self
            } else {
                self
            }
        }

        fn variable_sum(mut self: Testparser) -> Testparser {
            //plus sign, value, value (both ints or both floats), e.g. "+ 1 2" (1 + 2 = 3) or "+ 1.2 3.4" (1.2 + 3.4 = 4.6)
            let mut original_self = self.clone();
            let without_brackets = self
                .clone()
                .word("+ ")
                .chomp_clear()
                .first_success_of(
                    [Testparser::variable_sum, Testparser::float, Testparser::int].to_vec(),
                )
                .word(" ")
                .chomp_clear()
                .first_success_of(
                    [Testparser::variable_sum, Testparser::float, Testparser::int].to_vec(),
                );

            let with_brackets = self
                .clone()
                .word("(+ ")
                .chomp_clear()
                .first_success_of(
                    [Testparser::variable_sum, Testparser::float, Testparser::int].to_vec(),
                )
                .word(" ")
                .chomp_clear()
                .first_success_of(
                    [Testparser::variable_sum, Testparser::float, Testparser::int].to_vec(),
                )
                .word(")");
            if without_brackets.success {
                self = without_brackets;
            } else if with_brackets.success {
                self = with_brackets;
            } else {
                original_self.success = false;
                return original_self;
            }

            let mut el = TestparserElement::new();
            let variable1_el = self.output[self.output.len() - 2].clone();
            let variable2_el = self.output[self.output.len() - 1].clone();
            //check both values have the same element type
            match (variable1_el.el_type, variable2_el.el_type) {
                (Some(el1_type), Some(el2_type)) => {
                    if el1_type == el2_type {
                        match el1_type {
                            //if it's an int set the i64 of the new element to the sum of the 2 ints
                            TestparserElementType::Int64 => {
                                match (variable1_el.i64, variable2_el.i64) {
                                    (Some(val1), Some(val2)) => {
                                        el.i64 = Some(val1 + val2);
                                    }
                                    (_, _) => (),
                                }
                            }
                            _ => match (variable1_el.float64, variable2_el.float64) {
                                (Some(val1), Some(val2)) => {
                                    el.float64 = Some(val1 + val2);
                                }
                                (_, _) => (),
                            },
                        }
                        el.el_type = Some(el1_type);
                        self.output.remove(self.output.len() - 1);
                        self.output.remove(self.output.len() - 1);
                        self.output.push(el);
                        self.chomp = "".to_string();
                        self
                    } else {
                        self
                    }
                }
                (_, _) => self,
            }
        }
    }

    #[test]
    fn test_variable_sum() {
        //not a valid variable sum
        let mut testparser = Testparser::new(" + test 1");
        let result = testparser.clone().variable_sum();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, " + test 1");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, false);

        //short int plus short int, with optional brackets
        testparser = Testparser::new("(+ 1 2)");
        let result = testparser.clone().variable_sum();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(TestparserElementType::Int64));
        assert_eq!(result.output[0].i64, Some(3));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //short int plus short int
        testparser = Testparser::new("+ 1 2");
        let result = testparser.clone().variable_sum();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(TestparserElementType::Int64));
        assert_eq!(result.output[0].i64, Some(3));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //long int plus long int
        testparser = Testparser::new("+ 11111 22222");
        let result = testparser.clone().variable_sum();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(TestparserElementType::Int64));
        assert_eq!(result.output[0].i64, Some(33333));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //long int plus negative long int
        testparser = Testparser::new("+ 11111 -22222");
        let result = testparser.clone().variable_sum();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(TestparserElementType::Int64));
        assert_eq!(result.output[0].i64, Some(-11111));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //short float plus short float
        testparser = Testparser::new("+ 1.1 2.2");
        let result = testparser.clone().variable_sum();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(
            result.output[0].el_type,
            Some(TestparserElementType::Float64)
        );
        println!("************{:?}", result);
        assert_eq!(result.output[0].float64, Some(3.3000000000000003)); // yikes, floats
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //long float plus long float
        testparser = Testparser::new("+ 11111.11111 22222.22222");
        let result = testparser.clone().variable_sum();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(
            result.output[0].el_type,
            Some(TestparserElementType::Float64)
        );
        assert_eq!(result.output[0].float64, Some(33333.33333));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //long float plus negative long float
        testparser = Testparser::new("+ 11111.11111 -22222.22222");
        let result = testparser.clone().variable_sum();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(
            result.output[0].el_type,
            Some(TestparserElementType::Float64)
        );
        assert_eq!(result.output[0].float64, Some(-11111.11111));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);
    }
    #[test]
    fn test_variable_assign() {
        //not a variable assignment
        let mut testparser = Testparser::new(" = x 1");
        let result = testparser.clone().variable_assign();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, " = x 1");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, false);

        //"= x (+ 1 (+ 2 (+ 3 4)))", i.e. x = 1 + (2 + (3 + 4))
        //as below with brackets notation
        testparser = Testparser::new("= x (+ 1 (+ 2 (+ 3 4)))");
        let result = testparser.clone().variable_assign();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(
            result.output[0].el_type,
            Some(TestparserElementType::Variable)
        );
        assert_eq!(result.output[0].variable, Some("x".to_string()));
        assert_eq!(result.output[0].i64, Some(10));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //"= x + 1 + 2 + 3 4", i.e. x = 1 + (2 + (3 + 4))
        //short name variable assignment to sum of 2 short ints, where the second is 2 nested sums of 2 short ints
        testparser = Testparser::new("= x + 1 + 2 + 3 4");
        let result = testparser.clone().variable_assign();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(
            result.output[0].el_type,
            Some(TestparserElementType::Variable)
        );
        assert_eq!(result.output[0].variable, Some("x".to_string()));
        assert_eq!(result.output[0].i64, Some(10));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //"= x + + 1 2 + 3 4", i.e. x = (1 + 2) + (3 + 4))
        //short name variable assignment to sum of 2 short ints, where the second is 2 nested sums of 2 short ints, different format
        testparser = Testparser::new("= x + + 1 2 + 3 4");
        let result = testparser.clone().variable_assign();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(
            result.output[0].el_type,
            Some(TestparserElementType::Variable)
        );
        assert_eq!(result.output[0].variable, Some("x".to_string()));
        assert_eq!(result.output[0].i64, Some(10));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //"= x + 1 + 2 3", i.e. x = 1 + (2 + 3)
        //short name variable assignment to sum of 2 short ints, where the second is a sum of 2 short ints
        testparser = Testparser::new("= x + 1 + 2 3");
        let result = testparser.clone().variable_assign();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(
            result.output[0].el_type,
            Some(TestparserElementType::Variable)
        );
        assert_eq!(result.output[0].variable, Some("x".to_string()));
        assert_eq!(result.output[0].i64, Some(6));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //short name variable assignment to sum of 2 short ints
        testparser = Testparser::new("= x + 1 2");
        let result = testparser.clone().variable_assign();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(
            result.output[0].el_type,
            Some(TestparserElementType::Variable)
        );
        assert_eq!(result.output[0].variable, Some("x".to_string()));
        assert_eq!(result.output[0].i64, Some(3));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //short name variable assignment to sum of 2 long floats
        testparser = Testparser::new("= x + 11111.11111 22222.22222");
        let result = testparser.clone().variable_assign();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(
            result.output[0].el_type,
            Some(TestparserElementType::Variable)
        );
        assert_eq!(result.output[0].variable, Some("x".to_string()));
        assert_eq!(result.output[0].float64, Some(33333.33333));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //short name variable assignment to short int
        testparser = Testparser::new("= x 1");
        let result = testparser.clone().variable_assign();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(
            result.output[0].el_type,
            Some(TestparserElementType::Variable)
        );
        assert_eq!(result.output[0].variable, Some("x".to_string()));
        assert_eq!(result.output[0].i64, Some(1));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //long name variable with grapheme assignment to long negative int
        testparser = Testparser::new("= éxample_long_variable_name -123456");
        let result = testparser.clone().variable_assign();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(
            result.output[0].el_type,
            Some(TestparserElementType::Variable)
        );
        assert_eq!(
            result.output[0].variable,
            Some("éxample_long_variable_name".to_string())
        );
        assert_eq!(result.output[0].i64, Some(-123456));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //short name variable assignment to short float
        testparser = Testparser::new("= x 1.2");
        let result = testparser.clone().variable_assign();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(
            result.output[0].el_type,
            Some(TestparserElementType::Variable)
        );
        assert_eq!(result.output[0].variable, Some("x".to_string()));
        assert_eq!(result.output[0].float64, Some(1.2));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //short name variable assignment to long negative float
        testparser = Testparser::new("= x 1.2");
        let result = testparser.clone().variable_assign();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(
            result.output[0].el_type,
            Some(TestparserElementType::Variable)
        );
        assert_eq!(result.output[0].variable, Some("x".to_string()));
        assert_eq!(result.output[0].float64, Some(1.2));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);
    }

    #[test]
    fn test_variable() {
        //not a variable
        let mut testparser = Testparser::new(" x = 1");
        let result = testparser.clone().variable();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, " x = 1");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, false);

        //short name variable
        testparser = Testparser::new("x = 1");
        let result = testparser.clone().variable();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "= 1");
        assert_eq!(result.output.len(), 1);
        assert_eq!(
            result.output[0].el_type,
            Some(TestparserElementType::Variable)
        );
        assert_eq!(result.output[0].variable, Some("x".to_string()));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //long name variable with grapheme
        testparser = Testparser::new("éxample_long_variable_name = 123.45");
        let result = testparser.clone().variable();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "= 123.45");
        assert_eq!(result.output.len(), 1);
        assert_eq!(
            result.output[0].el_type,
            Some(TestparserElementType::Variable)
        );
        assert_eq!(
            result.output[0].variable,
            Some("éxample_long_variable_name".to_string())
        );
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);
    }
    #[test]
    fn test_float() {
        //not a float
        let mut testparser = Testparser::new("a123.456");
        let result = testparser.clone().float();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "a123.456");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, false);

        //positive small float
        testparser = Testparser::new("12.34");
        let result = testparser.clone().float();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(
            result.output[0].el_type,
            Some(TestparserElementType::Float64)
        );
        assert_eq!(result.output[0].float64, Some(12.34));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //positive large float
        testparser = Testparser::new("123456.78");
        let result = testparser.clone().float();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(
            result.output[0].el_type,
            Some(TestparserElementType::Float64)
        );
        assert_eq!(result.output[0].float64, Some(123456.78));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //negative float
        testparser = Testparser::new("-123456.78");
        let result = testparser.clone().float();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(
            result.output[0].el_type,
            Some(TestparserElementType::Float64)
        );
        assert_eq!(result.output[0].float64, Some(-123456.78));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);
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
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //positive large int
        testparser = Testparser::new("123456");
        let result = testparser.clone().int();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(TestparserElementType::Int64));
        assert_eq!(result.output[0].i64, Some(123456));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //negative int
        testparser = Testparser::new("-123456");
        let result = testparser.clone().int();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(TestparserElementType::Int64));
        assert_eq!(result.output[0].i64, Some(-123456));
        assert_eq!(result.chomp, "");
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
