#[cfg(test)]
mod tests {
    //Referring to https://package.elm-lang.org/packages/elm/parser/latest/Parser

    //el_int
    //el_float
    //el_var
    //fn fn_var_assign (=)
    //fn fn_var_sum (+)

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
    any_prim_char
    whitespace_prim_char
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
        var: Option<String>,
    }

    impl TestparserElement {
        fn new() -> TestparserElement {
            TestparserElement {
                el_type: None,
                i64: None,
                float64: None,
                var: None,
            }
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    enum TestparserElementType {
        Int64,
        Float64,
        Var,
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

        //MAIN PARSER
        fn parse(mut self: Testparser) -> Testparser {
            while self.success && self.input_remaining.len() > 0 {
                self = self.combi_first_success_of([Testparser::fn_var_assign].to_vec());
            }
            self
        }

        //PRIMITIVES

        fn prim_word(mut self: Testparser, expected: &str) -> Testparser {
            //any series of prim_characters, in the "expected" string
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

        fn prim_char(mut self: Testparser) -> Testparser {
            //a prim_character, excluding ' '(space)
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

        fn prim_digit(mut self: Testparser) -> Testparser {
            //a single prim_digit 0,1,2,3,4,5,6,7,8,9
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

        fn prim_eol(mut self: Testparser) -> Testparser {
            //\r\n or \n
            if self.success {
                let newline1 = self
                    .clone()
                    .combi_one_or_more_of(|s| Testparser::prim_word(s, "\r\n"));
                let newline2 = self
                    .clone()
                    .combi_one_or_more_of(|s| Testparser::prim_word(s, "\n"));
                if newline1.success {
                    newline1
                } else if newline2.success {
                    newline2
                } else {
                    self.success = false;
                    self
                }
            } else {
                self
            }
        }

        fn prim_eof(mut self: Testparser) -> Testparser {
            //check for an empty string...

            if self.success && self.input_remaining.len() == 0 {
                self
            } else {
                self.success = false;
                self
            }
        }

        fn prim_eol_or_eof(self: Testparser) -> Testparser {
            if self.success {
                self.combi_first_success_of([Testparser::prim_eol, Testparser::prim_eof].to_vec())
            } else {
                self
            }
        }

        //COMBINATORS

        fn combi_one_or_more_of<F>(mut self: Testparser, func: F) -> Testparser
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

        fn combi_zero_or_more_of<F>(mut self: Testparser, func: F) -> Testparser
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

        fn combi_optional<F>(mut self: Testparser, func: F) -> Testparser
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

        fn combi_first_success_of<F>(mut self: Testparser, funcs: Vec<F>) -> Testparser
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

        fn el_int(mut self: Testparser) -> Testparser {
            //integer number, e.g. 12 or -123456
            if self.success {
                self = self
                    .combi_optional(|s: Testparser| Testparser::prim_word(s, "-"))
                    .combi_one_or_more_of(Testparser::prim_digit);

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

        fn el_float(mut self: Testparser) -> Testparser {
            //floating point number, e.g. 12.34 or -123.45
            if self.success {
                self = self
                    .combi_optional(|s: Testparser| Testparser::prim_word(s, "-"))
                    .combi_one_or_more_of(Testparser::prim_digit)
                    .prim_word(".")
                    .combi_one_or_more_of(Testparser::prim_digit);

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

        fn el_var(mut self: Testparser) -> Testparser {
            //el_var name of prim_chars followed by a space, e.g. "x" or "lö̲ng_variablé_name"
            self = self
                .combi_one_or_more_of(Testparser::prim_char)
                .prim_word(" ");
            if self.success {
                let mut el = TestparserElement::new();
                let chomp = self.clone().chomp;
                let el_var = chomp[..(chomp.len() - 1)].to_string();
                el.el_type = Some(TestparserElementType::Var);
                el.var = Some(el_var);
                self.output.push(el);
                self.chomp = "".to_string();
                self
            } else {
                self
            }
        }

        //FUNCTIONS

        fn fn_var_assign(mut self: Testparser) -> Testparser {
            //equals sign, el_var name, value (test using el_int for now), e.g. "= x 1" (x equals 1)
            self = self
                .prim_word("= ")
                .chomp_clear()
                .el_var()
                .combi_first_success_of(
                    [
                        Testparser::fn_var_sum,
                        //el_float first so the number before . is not thought of as an el_int
                        Testparser::el_float,
                        Testparser::el_int,
                    ]
                    .to_vec(),
                )
                .prim_eol_or_eof();
            if self.success {
                let mut el = TestparserElement::new();
                let variable_el = self.output[self.output.len() - 2].clone();
                let value_el = self.output[self.output.len() - 1].clone();
                match value_el.el_type {
                    Some(TestparserElementType::Int64) => el.i64 = value_el.i64,
                    _ => el.float64 = value_el.float64,
                }
                el.el_type = Some(TestparserElementType::Var);
                el.var = variable_el.var;
                self.output.remove(self.output.len() - 1);
                self.output.remove(self.output.len() - 1);
                self.output.push(el);
                self.chomp = "".to_string();
                self
            } else {
                self
            }
        }

        fn fn_var_sum(mut self: Testparser) -> Testparser {
            //plus sign, value, value (both ints or both floats), e.g. "+ 1 2" (1 + 2 = 3) or "+ 1.2 3.4" (1.2 + 3.4 = 4.6)
            let mut original_self = self.clone();
            let without_brackets = self
                .clone()
                .prim_word("+ ")
                .chomp_clear()
                .combi_first_success_of(
                    [
                        Testparser::fn_var_sum,
                        Testparser::el_float,
                        Testparser::el_int,
                    ]
                    .to_vec(),
                )
                .prim_word(" ")
                .chomp_clear()
                .combi_first_success_of(
                    [
                        Testparser::fn_var_sum,
                        Testparser::el_float,
                        Testparser::el_int,
                    ]
                    .to_vec(),
                );

            let with_brackets = self
                .clone()
                .prim_word("(+ ")
                .chomp_clear()
                .combi_first_success_of(
                    [
                        Testparser::fn_var_sum,
                        Testparser::el_float,
                        Testparser::el_int,
                    ]
                    .to_vec(),
                )
                .prim_word(" ")
                .chomp_clear()
                .combi_first_success_of(
                    [
                        Testparser::fn_var_sum,
                        Testparser::el_float,
                        Testparser::el_int,
                    ]
                    .to_vec(),
                )
                .prim_word(")");
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
                            //if it's an el_int set the i64 of the new element to the sum of the 2 ints
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
        //not a valid el_var sum
        let mut testparser = Testparser::new(" + test 1");
        let result = testparser.clone().fn_var_sum();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, " + test 1");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, false);

        //short el_int plus short el_int, with combi_optional brackets
        testparser = Testparser::new("(+ 1 2)");
        let result = testparser.clone().fn_var_sum();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(TestparserElementType::Int64));
        assert_eq!(result.output[0].i64, Some(3));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //short el_int plus short el_int
        testparser = Testparser::new("+ 1 2");
        let result = testparser.clone().fn_var_sum();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(TestparserElementType::Int64));
        assert_eq!(result.output[0].i64, Some(3));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //long el_int plus long el_int
        testparser = Testparser::new("+ 11111 22222");
        let result = testparser.clone().fn_var_sum();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(TestparserElementType::Int64));
        assert_eq!(result.output[0].i64, Some(33333));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //long el_int plus negative long el_int
        testparser = Testparser::new("+ 11111 -22222");
        let result = testparser.clone().fn_var_sum();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(TestparserElementType::Int64));
        assert_eq!(result.output[0].i64, Some(-11111));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //short el_float plus short el_float
        testparser = Testparser::new("+ 1.1 2.2");
        let result = testparser.clone().fn_var_sum();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(
            result.output[0].el_type,
            Some(TestparserElementType::Float64)
        );
        assert_eq!(result.output[0].float64, Some(3.3000000000000003)); // yikes, floats
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //long el_float plus long el_float
        testparser = Testparser::new("+ 11111.11111 22222.22222");
        let result = testparser.clone().fn_var_sum();
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

        //long el_float plus negative long el_float
        testparser = Testparser::new("+ 11111.11111 -22222.22222");
        let result = testparser.clone().fn_var_sum();
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
    fn test_multiple_variable_assign() {
        let testparser = Testparser::new("= x + 1 2\r\n= y + 3 4\r\n= z + 5.0 6.0");
        let result = testparser.clone().parse();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 3);
        assert_eq!(result.output[0].el_type, Some(TestparserElementType::Var));
        assert_eq!(result.output[0].var, Some("x".to_string()));
        assert_eq!(result.output[0].i64, Some(3));
        assert_eq!(result.output[1].el_type, Some(TestparserElementType::Var));
        assert_eq!(result.output[1].var, Some("y".to_string()));
        assert_eq!(result.output[1].i64, Some(7));
        assert_eq!(result.output[2].el_type, Some(TestparserElementType::Var));
        assert_eq!(result.output[2].var, Some("z".to_string()));
        assert_eq!(result.output[2].float64, Some(11.0));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);
    }
    #[test]
    fn test_variable_assign() {
        //not a el_var assignment
        let mut testparser = Testparser::new(" = x 1");
        let result = testparser.clone().fn_var_assign();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, " = x 1");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, false);

        //"= x (+ 1 (+ 2 (+ 3 4)))", i.e. x = 1 + (2 + (3 + 4))
        //as below with brackets notation
        testparser = Testparser::new("= x (+ 1 (+ 2 (+ 3 4)))");
        let result = testparser.clone().fn_var_assign();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(TestparserElementType::Var));
        assert_eq!(result.output[0].var, Some("x".to_string()));
        assert_eq!(result.output[0].i64, Some(10));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //"= x + 1 + 2 + 3 4", i.e. x = 1 + (2 + (3 + 4))
        //short name el_var assignment to sum of 2 short ints, where the second is 2 nested sums of 2 short ints
        testparser = Testparser::new("= x + 1 + 2 + 3 4");
        let result = testparser.clone().fn_var_assign();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(TestparserElementType::Var));
        assert_eq!(result.output[0].var, Some("x".to_string()));
        assert_eq!(result.output[0].i64, Some(10));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //"= x + + 1 2 + 3 4", i.e. x = (1 + 2) + (3 + 4))
        //short name el_var assignment to sum of 2 short ints, where the second is 2 nested sums of 2 short ints, different format
        testparser = Testparser::new("= x + + 1 2 + 3 4");
        let result = testparser.clone().fn_var_assign();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(TestparserElementType::Var));
        assert_eq!(result.output[0].var, Some("x".to_string()));
        assert_eq!(result.output[0].i64, Some(10));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //"= x + 1 + 2 3", i.e. x = 1 + (2 + 3)
        //short name el_var assignment to sum of 2 short ints, where the second is a sum of 2 short ints
        testparser = Testparser::new("= x + 1 + 2 3");
        let result = testparser.clone().fn_var_assign();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(TestparserElementType::Var));
        assert_eq!(result.output[0].var, Some("x".to_string()));
        assert_eq!(result.output[0].i64, Some(6));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //short name el_var assignment to sum of 2 short ints
        testparser = Testparser::new("= x + 1 2");
        let result = testparser.clone().fn_var_assign();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(TestparserElementType::Var));
        assert_eq!(result.output[0].var, Some("x".to_string()));
        assert_eq!(result.output[0].i64, Some(3));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //short name el_var assignment to sum of 2 long floats
        testparser = Testparser::new("= x + 11111.11111 22222.22222");
        let result = testparser.clone().fn_var_assign();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(TestparserElementType::Var));
        assert_eq!(result.output[0].var, Some("x".to_string()));
        assert_eq!(result.output[0].float64, Some(33333.33333));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //short name el_var assignment to short el_int
        testparser = Testparser::new("= x 1");
        let result = testparser.clone().fn_var_assign();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(TestparserElementType::Var));
        assert_eq!(result.output[0].var, Some("x".to_string()));
        assert_eq!(result.output[0].i64, Some(1));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //short name el_var assignment to short el_int with newlines
        testparser = Testparser::new("= x 1\r\n\r\n\r\n");
        let result = testparser.clone().fn_var_assign();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(TestparserElementType::Var));
        assert_eq!(result.output[0].var, Some("x".to_string()));
        assert_eq!(result.output[0].i64, Some(1));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //long name el_var with grapheme assignment to long negative el_int
        testparser = Testparser::new("= éxample_long_variable_name -123456");
        let result = testparser.clone().fn_var_assign();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(TestparserElementType::Var));
        assert_eq!(
            result.output[0].var,
            Some("éxample_long_variable_name".to_string())
        );
        assert_eq!(result.output[0].i64, Some(-123456));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //short name el_var assignment to short el_float
        testparser = Testparser::new("= x 1.2");
        let result = testparser.clone().fn_var_assign();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(TestparserElementType::Var));
        assert_eq!(result.output[0].var, Some("x".to_string()));
        assert_eq!(result.output[0].float64, Some(1.2));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //short name el_var assignment to long negative el_float
        testparser = Testparser::new("= x 1.2");
        let result = testparser.clone().fn_var_assign();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(TestparserElementType::Var));
        assert_eq!(result.output[0].var, Some("x".to_string()));
        assert_eq!(result.output[0].float64, Some(1.2));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);
    }

    #[test]
    fn test_variable() {
        //not a el_var
        let mut testparser = Testparser::new(" x = 1");
        let result = testparser.clone().el_var();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, " x = 1");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, false);

        //short name el_var
        testparser = Testparser::new("x = 1");
        let result = testparser.clone().el_var();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "= 1");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(TestparserElementType::Var));
        assert_eq!(result.output[0].var, Some("x".to_string()));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //long name el_var with grapheme
        testparser = Testparser::new("éxample_long_variable_name = 123.45");
        let result = testparser.clone().el_var();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "= 123.45");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(TestparserElementType::Var));
        assert_eq!(
            result.output[0].var,
            Some("éxample_long_variable_name".to_string())
        );
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);
    }
    #[test]
    fn test_float() {
        //not a el_float
        let mut testparser = Testparser::new("a123.456");
        let result = testparser.clone().el_float();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "a123.456");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, false);

        //positive small el_float
        testparser = Testparser::new("12.34");
        let result = testparser.clone().el_float();
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

        //positive large el_float
        testparser = Testparser::new("123456.78");
        let result = testparser.clone().el_float();
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

        //negative el_float
        testparser = Testparser::new("-123456.78");
        let result = testparser.clone().el_float();
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
        //not an el_int
        let mut testparser = Testparser::new("a123");
        let result = testparser.clone().el_int();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "a123");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, false);

        //positive small el_int
        testparser = Testparser::new("12");
        let result = testparser.clone().el_int();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(TestparserElementType::Int64));
        assert_eq!(result.output[0].i64, Some(12));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //positive large el_int
        testparser = Testparser::new("123456");
        let result = testparser.clone().el_int();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(TestparserElementType::Int64));
        assert_eq!(result.output[0].i64, Some(123456));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //negative el_int
        testparser = Testparser::new("-123456");
        let result = testparser.clone().el_int();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(TestparserElementType::Int64));
        assert_eq!(result.output[0].i64, Some(-123456));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);
    }

    #[test]
    fn test_combi_optional() {
        let mut testparser = Testparser::new("a123Test");
        let result = testparser.clone().combi_optional(Testparser::prim_char);
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "123Test");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "a");
        assert_eq!(result.success, true);

        testparser = Testparser::new("a123Test");
        let result = testparser
            .clone()
            .combi_zero_or_more_of(Testparser::prim_digit);
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "a123Test");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);
    }

    #[test]
    fn test_combi_zero_or_more_of() {
        let mut testparser = Testparser::new("a123Test");
        let result = testparser
            .clone()
            .combi_zero_or_more_of(Testparser::prim_digit);
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "a123Test");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        testparser = Testparser::new("123Test");
        let result = testparser
            .clone()
            .combi_zero_or_more_of(Testparser::prim_digit);
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "Test");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "123");
        assert_eq!(result.success, true);
    }
    #[test]
    fn test_combi_one_or_more_of() {
        let mut testparser = Testparser::new("a123Test");
        let result = testparser
            .clone()
            .combi_one_or_more_of(Testparser::prim_digit);
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "a123Test");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, false);

        testparser = Testparser::new("123Test");
        let result = testparser
            .clone()
            .combi_one_or_more_of(Testparser::prim_digit);
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "Test");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "123");
        assert_eq!(result.success, true);
    }

    #[test]
    fn test_multiple_parsers() {
        let testparser = Testparser::new("1Test");
        let result = testparser.clone().prim_digit().prim_word("Te");
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "st");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "1Te");
        assert_eq!(result.success, true);
    }
    #[test]
    fn test_prim_eof_or_eol() {
        //not eof or eol
        let testparser = Testparser::new("1");
        let result = testparser.clone().prim_eol_or_eof();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "1");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, false);

        //eof
        let testparser = Testparser::new("");
        let result = testparser.clone().prim_eol_or_eof();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //single eol1
        let testparser = Testparser::new("\n");
        let result = testparser.clone().prim_eol_or_eof();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "\n");
        assert_eq!(result.success, true);

        //single eol2
        let testparser = Testparser::new("\r\n");
        let result = testparser.clone().prim_eol_or_eof();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "\r\n");
        assert_eq!(result.success, true);

        //multiple eol1
        let testparser = Testparser::new("\n\n\n\n");
        let result = testparser.clone().prim_eol_or_eof();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "\n\n\n\n");
        assert_eq!(result.success, true);

        //multiple eol2
        let testparser = Testparser::new("\r\n\r\n\r\n\r\n");
        let result = testparser.clone().prim_eol_or_eof();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "\r\n\r\n\r\n\r\n");
        assert_eq!(result.success, true);
    }

    #[test]
    fn test_prim_eof() {
        //not eof
        let testparser = Testparser::new("1");
        let result = testparser.clone().prim_eof();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "1");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, false);

        //eof
        let testparser = Testparser::new("");
        let result = testparser.clone().prim_eof();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);
    }

    #[test]
    fn test_prim_eol() {
        //not an eol
        let testparser = Testparser::new("1");
        let result = testparser.clone().prim_eol();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "1");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, false);

        //single eol1
        let testparser = Testparser::new("\n");
        let result = testparser.clone().prim_eol();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "\n");
        assert_eq!(result.success, true);

        //single eol2
        let testparser = Testparser::new("\r\n");
        let result = testparser.clone().prim_eol();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "\r\n");
        assert_eq!(result.success, true);

        //multiple eol1
        let testparser = Testparser::new("\n\n\n\n");
        let result = testparser.clone().prim_eol();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "\n\n\n\n");
        assert_eq!(result.success, true);

        //multiple eol2
        let testparser = Testparser::new("\r\n\r\n\r\n\r\n");
        let result = testparser.clone().prim_eol();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "\r\n\r\n\r\n\r\n");
        assert_eq!(result.success, true);
    }

    #[test]
    fn test_prim_digit() {
        let testparser = Testparser::new("123Test");
        let result = testparser.clone().prim_digit().prim_digit().prim_digit();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "Test");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "123");
        assert_eq!(result.success, true);
    }
    #[test]
    fn test_prim_char() {
        let testparser = Testparser::new("Testing 123");
        let result = testparser
            .clone()
            .prim_char()
            .prim_char()
            .prim_char()
            .prim_char();
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "ing 123");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "Test");
        assert_eq!(result.success, true);
    }

    #[test]
    fn test_prim_word() {
        let testparser = Testparser::new("Testing 123");
        let result = testparser
            .clone()
            .prim_word("Test")
            .prim_word("ing")
            .prim_word(" ")
            .prim_word("123");
        assert_eq!(result.input_original, testparser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "Testing 123");
        assert_eq!(result.success, true);
    }
}
