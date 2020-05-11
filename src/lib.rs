use colored::*;
use std::cmp::PartialEq;
use unicode_segmentation::UnicodeSegmentation;

///This is a toy parser/compiler loosely taking inspiration from [Elm Parser](https://package.elm-lang.org/packages/elm/parser/latest/Parser) with the following methods so far...
///
///The goal is to have fun seeing if I can build a 'simple' parser syntax, similar in style Elm Parser, using the power of Rust, but without the visual/mental overhead of standard Rust code.
///
///Essentially, building a parser, to parse a new toy parser language
///
///# Examples of language syntax
///
/** ### Primitives (terse syntax is (mostly) 1 character)
| Syntax | Parser Name                                  | Example Input String | Terse Syntax | (or) Written Syntax        | Example Result           |
|--------|----------------------------------------------|----------------------|--------------|----------------------------|--------------------------|
| >      | [prim_next](#method.prim_next)               | `1234`               | `>>`         | `(next)`                   | `12` (in Parser.chomp)   |
| "      | [prim_quote](#method.prim_quote)             | `"test"`             | `"`          | `(quote)`                  | `\"` (in Parser.chomp)   |
| 'word' | [prim_word](#method.prim_word)               | `testing`            | `'test'`     | `(word test)`              | `test` (in Parser.chomp) |
| @      | [prim_char](#method.prim_char)               | `testing`            | `@@@@`       | `(char)(char)(char)(char)` | `test` (in Parser.chomp) |
| \#     | [prim_digit](#method.prim_digit)             | `1234`               | `##`         | `(0-9)(0-9)`               | `12` (in Parser.chomp)   |
| ,      | [prim_eol](#method.prim_eol)                 | ` `                  | `,`          | `(eols)`                   | true (in Parser.success) |
|        |                                              | `second line`        |              |                            |                          |
| .      | [prim_eof](#method.prim_eof)                 | ` `                  | `.`          | `(eof)`                    | true (in Parser.success) |
| ;      | [prim_eols_or_eof](#method.prim_eols_or_eof) | ` `                  | `;`          | `(eolseof)`                | true (in Parser.success) |
|        |                                              | ` `                  |              |                            |                          |
|        |                                              | `third line`         |              |                            |                          |

### Combinators (terse syntax is 2 characters)
| Syntax | Parser Name                                                        | Example Input String | Terse Syntax | (or) Written Syntax         | Example Result             |
|--------|--------------------------------------------------------------------|----------------------|--------------|-----------------------------|----------------------------|
| 1+     | [combi_one_or_more_of](#method.combi_one_or_more_of)               | `1234test`           | `1+#`        | `(one+ (0-9))`              | `1234` (in Parser.chomp)   |
| 0+     | [combi_zero_or_more_of](#method.combi_zero_or_more_of)             | `"test1234"`         | `0+@`        | `(zero+ (char))`            | `test` (in Parser.chomp)   |
| !+     | [combi_until_first_do_second](#method.combi_until_first_do_second) | `testing`            | `!+'i'@`     | `(two!one (word i) (char))` | `test` (in Parser.chomp)   |
| ??     | [combi_optional](#method.combi_optional)                           | `1test`              | `??#`        | `(option (0-9))`            | `true` (in Parser.success) |
| []     | [combi_first_success_of](#method.combi_first_success_of)           | `1234test`           | `[# @]`      | `(0-9)(char)`               | `1` (in Parser.chomp)      |

### Elements (terse syntax is 4 characters)
| Syntax | Parser Name                   | Example Input String | Terse Syntax | (or) Written Syntax | Example Result                    |
|--------|-------------------------------|----------------------|--------------|---------------------|-----------------------------------|
| $str   | [el_str](#method.el_str)      | `"1234"`             | `$str`       | `(el_str)`          | `1234` (string element no name)   |
| $int   | [el_int](#method.el_int)      | `-1234`              | `$int`       | `(el_int)`          | `-1234` (integer element no name) |
| $flt   | [el_float](#method.el_float)  | `-123.45`            | `$flt`       | `(el_flt)`          | `-123.45` (float element no name) |
| $var   | [el_var](#method.el_var)      | `= x "test"`         | `$var`       | `(el_var)`          | `test` (string element named `x`) |

**/
/// ### Functions (perhaps these should be in userland?)
///[fn_var_assign (=)](#method.fn_var_assign),
///
///[fn_var_sum (+)](#method.fn_var_sum)
///<br /><br />
///Parser is initialised once using [new](#method.new) for each string you wish to parse.<br />
///Then it is passed through all the parser functions you have defined<br />
///This is the current 'state' of the parser at any one time during its passage through all the parser functions
///- input_original: always contains the initial string supplied to [new](#method.new)
///- input_remaining: is the current state of the remaining string to be parsed, as it passes through each parser function
///- output: is a vec of ParserElements, generated by your parser functions
///- chomp: is the sub-string built up by a subgroup of the current parser functions.<br />
///  It can be cleared manually with [chomp_clear](#method.chomp_clear) and is usually used to build some fragment of a string for e.g. a variable name
///- success: is set to true or false by the current parser function. Currently, if a fail occurs, it is passed through all functions until the last one<br />
///  (TODO) use Results, and Panic during main parser functions
#[derive(Debug, Clone)]
pub struct Parser {
    input_original: String,
    input_remaining: String,
    output: Vec<ParserElement>,
    chomp: String,
    chomping: bool,
    success: bool,
    display_errors: bool,
}

#[derive(Debug, Clone)]
///Usually the end result of parsing a complete individual 'thing' within the whole parsed output<br /><br />
///
///A completed parse, should result in a vec of ParserElements in the parser.output<br /><br />
///
///All attributes are Options defining a single instance of a thing<br />
/// - el_type: the type of thing it is<br />
/// - what 'value' it should have depending on which are populated, here there are only 2 types<br />
///   - in64<br />
///   - float64<br />
/// - var_name: a string for the name if it is a variable
pub struct ParserElement {
    el_type: Option<ParserElementType>,
    int64: Option<i64>,
    float64: Option<f64>,
    string: Option<String>,
    var_name: Option<String>,
}

impl ParserElement {
    pub fn new() -> ParserElement {
        ParserElement {
            el_type: None,
            int64: None,
            float64: None,
            string: None,
            var_name: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParserElementType {
    Int64,
    Float64,
    Var,
    Str,
}

/// ## Main Methods
impl Parser {
    ///Initialises a new parser with the string you wish to parse
    pub fn new(input_string: &str) -> Parser {
        Parser {
            input_original: input_string.to_string(),
            input_remaining: input_string.to_string(),
            chomp: "".to_string(),
            chomping: true,
            output: Vec::<ParserElement>::new(),
            success: true,
            display_errors: true,
        }
    }

    ///Defines the parser to run, then runs it on the initialised parser from new
    pub fn parse(mut self: Parser) -> Parser {
        while self.success && self.input_remaining.len() > 0 {
            self = self.combi_first_success_of(&[Parser::fn_var_assign].to_vec());
        }
        self
    }

    ///Initialises and runs the supplied parser functions (as a closure) on a supplied string
    ///
    ///### Example
    ///Using the parser to parse the string '= x 123' which is meant to describe assigning the value 123 to variable x
    ///'=' is the assign function which takes two parameters (variable name<String>, value<Integer>)
    ///```
    ///let my_parser = |p| rust_learning_parser_combinators::Parser::fn_var_assign(p);
    ///let string_to_parse = "= x 123";
    ///let parse_result = rust_learning_parser_combinators::Parser::new2(string_to_parse, my_parser);
    ///println!("{:?}",parse_result);
    ///```

    ///You can use a combinator to check for multiple options, e.g. the second line adds the 'sum' function taking two parameters, x and 456, and assigns that new value to x
    ///
    ///```
    ///let my_parser = |p| {
    ///        rust_learning_parser_combinators::Parser::combi_first_success_of(
    ///            p,
    ///            &[
    ///                rust_learning_parser_combinators::Parser::fn_var_assign,
    ///                rust_learning_parser_combinators::Parser::fn_var_sum,
    ///            ]
    ///            .to_vec(),
    ///        )
    ///    };
    ///let string_to_parse = "
    ///= x 123
    ///= x + x 456";
    ///let parse_result = rust_learning_parser_combinators::Parser::new2(string_to_parse, my_parser);
    ///println!("{:?}",parse_result);
    ///```

    pub fn new2<F>(input_string: &str, func: F) -> Parser
    where
        F: Fn(Parser) -> Parser,
    {
        let mut new_parser: Parser = Parser {
            input_original: input_string.to_string(),
            input_remaining: input_string.to_string(),
            chomp: "".to_string(),
            chomping: true,
            output: Vec::<ParserElement>::new(),
            success: true,
            display_errors: true,
        };
        //while new_parser.success {
        new_parser = func(new_parser);
        //}
        new_parser
    }

    pub fn display_error(self: &Parser, from: &str) {
        //only display a short 100 char excerpt of remaining string
        let mut length = self.input_remaining.len();
        let position = self.input_original.len() - length;
        if length > 100 {
            length = 100;
        }
        if self.display_errors {
            println!(
                "\r\n{}\r\n{} at {} position:{}\r\n{}\r\n{}\r\n{:?}\r\n{}",
                "vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv"
                    .yellow(),
                "Parser Error".yellow(),
                from.red(),
                position,
                self.input_remaining.get(0..length).unwrap(),
                "Current Parser state looks like this:".yellow(),
                self,
                "^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^"
                    .yellow(),
            );
        }
    }

    ///Clears the current `chomp` value back to an empty string
    pub fn chomp_clear(mut self: Parser) -> Parser {
        self.chomp = "".to_string();
        self
    }

    ///Finds a variable by name if the parser created it already<br/>
    ///Result...<br/>
    ///Ok(index of the found variable within the parser.output, and the variable[ParserElement](struct.ParserElement.html)<br/>
    ///Err("Not found")
    pub fn get_el_var(self: Parser, var_name: &str) -> Result<(usize, ParserElement), &str> {
        let mut found = Err("Not found");
        for (i, el) in self.output.iter().enumerate() {
            if el.el_type == Some(ParserElementType::Var)
                && el.var_name == Some(var_name.to_string())
            {
                found = Ok((i, el.clone()))
            }
        }
        found
    }
}

/// ## Parser primitives
/// they don't Panic at an error -  but can return an error in case you need to capture that for parsing in a [Parser Combinator](#parser-combinators)
impl Parser {
    ///Matches whatever the next character is, fails if eof
    pub fn prim_next(mut self: Parser) -> Parser {
        if self.success {
            self = self.prim_eof();
            if self.success {
                self.success = false;
                self
            } else {
                match self.clone().input_remaining.graphemes(true).next() {
                    Some(next) => {
                        self.input_remaining = self.input_remaining[next.len()..].to_string();
                        if self.chomping {
                            self.chomp += next;
                        };
                        self.success = true;
                        self
                    }
                    _ => {
                        self.display_error("prim_next");
                        self.success = false;
                        self
                    }
                }
            }
        } else {
            self
        }
    }

    pub fn prim_quote(mut self: Parser) -> Parser {
        let chomping_previous_flag_setting = self.chomping;
        self.chomping = false;
        self = self.prim_word("\"");
        self.chomping = chomping_previous_flag_setting;
        self
    }

    /// Matches any series of [prim_car](#method.prim_char) in the supplied 'expected' string
    /// Always succeeds
    pub fn prim_word(mut self: Parser, expected: &str) -> Parser {
        if self.success {
            match self.clone().input_remaining.get(0..expected.len()) {
                Some(next) if next == expected => {
                    self.input_remaining = self.input_remaining[expected.len()..].to_string();
                    if self.chomping {
                        self.chomp += next;
                    };
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

    /// Matches any unicode character except whitespace '&nbsp;'
    pub fn prim_char(mut self: Parser) -> Parser {
        if self.success {
            match self.clone().input_remaining.graphemes(true).next() {
                Some(next) => {
                    if next == " " {
                        self.display_error("prim_char");
                        self.success = false;
                        self
                    } else {
                        self.input_remaining = self.input_remaining[next.len()..].to_string();
                        if self.chomping {
                            self.chomp += next;
                        };
                        self.success = true;
                        self
                    }
                }
                _ => {
                    self.success = false;
                    self.display_error("prim_char");
                    self
                }
            }
        } else {
            self.display_error("prim_char");
            self
        }
    }

    /// Matches a single digit 0,1,2,3,4,5,6,7,8,9
    pub fn prim_digit(mut self: Parser) -> Parser {
        if self.success {
            match self.input_remaining.chars().next() {
                Some(next) if next.is_digit(10) => {
                    self.input_remaining = self.input_remaining[next.len_utf8()..].to_string();
                    if self.chomping {
                        self.chomp += next.encode_utf8(&mut [0; 1]);
                    };

                    self.success = true;
                    self
                }
                _ => {
                    self.success = false;
                    self.display_error("prim_digit");
                    self
                }
            }
        } else {
            self.display_error("prim_digit");
            self
        }
    }

    /// Matches [a combination of one or more of](#method.combi_one_or_more_of) a single \r\n or \n
    pub fn prim_eol(mut self: Parser) -> Parser {
        if self.success {
            let newline1 = self
                .clone()
                .combi_one_or_more_of(|s| Parser::prim_word(s, "\r\n"));
            let newline2 = self
                .clone()
                .combi_one_or_more_of(|s| Parser::prim_word(s, "\n"));
            if newline1.success {
                newline1
            } else if newline2.success {
                newline2
            } else {
                self.success = false;
                self.display_error("prim_eol");
                self
            }
        } else {
            self.display_error("prim_eol");
            self
        }
    }

    ///Matches if you've reached the end of the parsed string, i.e. check for an empty string at this stage of the parser...
    pub fn prim_eof(mut self: Parser) -> Parser {
        if self.success && self.input_remaining.len() == 0 {
            self
        } else {
            self.success = false;
            self.display_error("prim_eof");
            self
        }
    }

    ///Matches either (prim_eols)[#method.prim_eols] or (prim_eof)[#method.prim_eof]
    pub fn prim_eols_or_eof(mut self: Parser) -> Parser {
        if self.success {
            let display_errors_previous_flag_setting = self.display_errors;
            self.display_errors = false;
            self = self.combi_first_success_of(&[Parser::prim_eol, Parser::prim_eof].to_vec());
            if self.success {
                self.display_errors = display_errors_previous_flag_setting;
                self
            } else {
                self.display_error("prim_eols_or_eof");
                self.display_errors = display_errors_previous_flag_setting;
                self
            }
        } else {
            self
        }
    }
}
/// ## Parser combinators
/// they will (TODO) Panic at an error -  used to combine multiple [Parser primitives](#parser-primitives) or other [Parser combinators](#parser-combinators)
impl Parser {
    ///Matches either one, or multiple of any one parser or combinator of parsers
    pub fn combi_one_or_more_of<F>(mut self: Parser, func: F) -> Parser
    where
        F: Fn(Parser) -> Parser,
    {
        if self.success {
            let chomp = self.clone().chomp;
            let display_errors_previous_flag_setting = self.display_errors;
            self.display_errors = false;
            while self.success {
                self = func(self);
            }
            self.display_errors = display_errors_previous_flag_setting;
            if self.chomp == chomp {
                self.display_error("combi_one_or_more_of");
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
    ///Matches either zero, one or multiple of any [Parser primitives](#parser-primitives) or other [Parser combinators](#parser-combinators).<br />
    ///Beware, it will always succeed!
    pub fn combi_zero_or_more_of<F>(mut self: Parser, func: F) -> Parser
    where
        F: Fn(Parser) -> Parser,
    {
        if self.success {
            let display_errors_previous_flag_setting = self.display_errors;
            self.display_errors = false;
            while self.success {
                self = func(self)
            }
            self.display_errors = display_errors_previous_flag_setting;
            self.success = true;
            self
        } else {
            self
        }
    }

    ///Matches either zero, one or multiple of any [Parser primitives](#parser-primitives) or other [Parser combinators](#parser-combinators)<br />
    ///until it reaches the second supplied [Parser primitives](#parser-primitives) or other [Parser combinators](#parser-combinators)
    pub fn combi_until_first_do_second<F>(mut self: Parser, first_and_second: Vec<F>) -> Parser
    where
        F: Fn(Parser) -> Parser,
    {
        if self.success {
            let display_errors_previous_flag_setting = self.display_errors;
            self.display_errors = false;
            while self.success {
                self = Parser::combi_first_success_of(self, &first_and_second);
            }
            self.display_errors = display_errors_previous_flag_setting;
            self.success = true;
            self
        } else {
            self
        }
    }

    ///Matches either one or zero of any [Parser primitives](#parser-primitives) or other [Parser combinators](#parser-combinators).<br />
    ///Beware, it will always succeed!
    pub fn combi_optional<F>(mut self: Parser, func: F) -> Parser
    where
        F: Fn(Parser) -> Parser,
    {
        if self.success {
            self = func(self);
            self.success = true;
            self
        } else {
            self.display_error("combi_optional");
            self
        }
    }

    ///Tries to match one of the parsers supplied in an array (vec) of [Parser primitives](#parser-primitives) or other [Parser combinators](#parser-combinators).
    ///
    ///It matches in the order supplied
    pub fn combi_first_success_of<F>(mut self: Parser, funcs: &Vec<F>) -> Parser
    where
        F: Fn(Parser) -> Parser,
    {
        if self.success {
            for func in funcs {
                let mut new_self = self.clone();
                let display_errors_previous_flag_setting = self.display_errors;
                new_self.display_errors = false;
                new_self = func(new_self);
                new_self.display_errors = display_errors_previous_flag_setting;
                if new_self.success {
                    return new_self;
                }
            }
            self.display_error("combi_first_success_of");
            self.success = false;
            return self;
        } else {
            return self;
        };
    }
}

/// ## Parser Elements

impl Parser {
    ///string, e.g. "123" or "The quick brown fox jumps over the lazy dog"
    pub fn el_str(mut self: Parser) -> Parser {
        if self.success {
            let display_errors_previous_flag_setting = self.display_errors;
            self.display_errors = false;
            self = self
                .prim_quote()
                .combi_until_first_do_second([Parser::prim_quote, Parser::prim_next].to_vec());
            self.display_errors = display_errors_previous_flag_setting;
            if self.success {
                let mut el = ParserElement::new();
                let val = self.clone().chomp;
                el.el_type = Some(ParserElementType::Str);
                el.string = Some(val);
                self.output.push(el);
                self = self.chomp_clear();
                self
            } else {
                self.display_error("el_str");
                self
            }
        } else {
            self
        }
    }

    ///integer number, e.g. 12 or -123456
    pub fn el_int(mut self: Parser) -> Parser {
        if self.success {
            let display_errors_previous_flag_setting = self.display_errors;
            self.display_errors = false;
            self = self
                .combi_optional(|s: Parser| Parser::prim_word(s, "-"))
                .combi_one_or_more_of(Parser::prim_digit);
            self.display_errors = display_errors_previous_flag_setting;
            if self.success {
                let mut el = ParserElement::new();
                let val = self.clone().chomp.parse().unwrap();
                el.el_type = Some(ParserElementType::Int64);
                el.int64 = Some(val);
                self.output.push(el);
                self = self.chomp_clear();
                self
            } else {
                self.display_error("el_int");
                self
            }
        } else {
            self
        }
    }

    ///floating point number, e.g. 12.34 or -123.45
    pub fn el_float(mut self: Parser) -> Parser {
        if self.success {
            let display_errors_previous_flag_setting = self.display_errors;
            self.display_errors = false;
            self = self
                .combi_optional(|s: Parser| Parser::prim_word(s, "-"))
                .combi_one_or_more_of(Parser::prim_digit)
                .prim_word(".")
                .combi_one_or_more_of(Parser::prim_digit);
            self.display_errors = display_errors_previous_flag_setting;
            if self.success {
                let mut el = ParserElement::new();
                let val = self.clone().chomp.parse().unwrap();
                el.el_type = Some(ParserElementType::Float64);
                el.float64 = Some(val);
                self.output.push(el);
                self = self.chomp_clear();
                self
            } else {
                self.display_error("el_float");
                self
            }
        } else {
            self
        }
    }

    ///el_var name of prim_chars followed by a space, e.g. "x" or "lö̲ng_variablé_name"
    pub fn el_var(mut self: Parser) -> Parser {
        self = self.combi_one_or_more_of(Parser::prim_char).prim_word(" ");
        if self.success {
            let display_errors_previous_flag_setting = self.display_errors;
            self.display_errors = false;
            let mut el = ParserElement::new();
            let chomp = self.clone().chomp;
            let el_var = chomp[..(chomp.len() - 1)].to_string();
            el.el_type = Some(ParserElementType::Var);
            el.var_name = Some(el_var);
            self.output.push(el);
            self = self.chomp_clear();
            self.display_errors = display_errors_previous_flag_setting;
            self
        } else {
            self.display_error("el_var");
            self
        }
    }

    /// ## Parser Elements
    ///
    ///equals sign, el_var name, value (test using el_int for now), e.g. "= x 1" (x equals 1)
    pub fn fn_var_assign(mut self: Parser) -> Parser {
        let temp_self = self
            .clone()
            .prim_word("= ")
            .chomp_clear()
            .el_var()
            .combi_first_success_of(
                &[
                    Parser::fn_var_sum,
                    //el_float first so the number before . is not thought of as an el_int
                    Parser::el_float,
                    Parser::el_int,
                ]
                .to_vec(),
            )
            .prim_eols_or_eof();
        if temp_self.success {
            let mut el = ParserElement::new();
            let variable_el = temp_self.output[temp_self.output.len() - 2].clone();
            let value_el = temp_self.output[temp_self.output.len() - 1].clone();
            match value_el.el_type {
                Some(ParserElementType::Int64) => el.int64 = value_el.int64,
                Some(ParserElementType::Str) => el.string = value_el.string,
                _ => el.float64 = value_el.float64,
            }
            el.el_type = Some(ParserElementType::Var);
            el.var_name = variable_el.var_name;

            //println!("{:?}", temp_self);

            //temp_self.output.remove(self.output.len() - 1);
            //temp_self.output.remove(self.output.len() - 1);

            match el.var_name.clone() {
                Some(var_name) => {
                    let var_exists_result = self.clone().get_el_var(&var_name);
                    match var_exists_result {
                        Ok((index, mut existing_var)) => {
                            existing_var.int64 = el.int64;
                            existing_var.float64 = el.float64;
                            self.output[index] = existing_var;
                        }
                        _ => {
                            self.input_remaining = temp_self.input_remaining;
                            self.output.push(el);
                            return self;
                        }
                    }
                    self.input_remaining = temp_self.input_remaining;
                    self = self.chomp_clear();
                    self
                }
                _ => {
                    self.display_error("fn_var_assign - can't find var_name for some reason (should be impossible)");
                    self.input_remaining = temp_self.input_remaining;
                    return self;
                }
            }
        } else {
            self.display_error("fn_var_assign");
            temp_self
        }
    }

    ///plus sign, value, value (both ints or both floats), e.g. "+ 1 2" (1 + 2 = 3) or "+ 1.2 3.4" (1.2 + 3.4 = 4.6)
    pub fn fn_var_sum(mut self: Parser) -> Parser {
        let mut original_self = self.clone();
        let without_brackets = self
            .clone()
            .prim_word("+ ")
            .chomp_clear()
            .combi_first_success_of(
                &[Parser::fn_var_sum, Parser::el_float, Parser::el_int].to_vec(),
            )
            .prim_word(" ")
            .chomp_clear()
            .combi_first_success_of(
                &[Parser::fn_var_sum, Parser::el_float, Parser::el_int].to_vec(),
            );

        let with_brackets = self
            .clone()
            .prim_word("(+ ")
            .chomp_clear()
            .combi_first_success_of(
                &[Parser::fn_var_sum, Parser::el_float, Parser::el_int].to_vec(),
            )
            .prim_word(" ")
            .chomp_clear()
            .combi_first_success_of(
                &[Parser::fn_var_sum, Parser::el_float, Parser::el_int].to_vec(),
            )
            .prim_word(")");

        if without_brackets.success {
            self = without_brackets;
        } else if with_brackets.success {
            self = with_brackets;
        } else {
            original_self.display_error("fn_var_sum");
            original_self.success = false;
            return original_self;
        }

        let mut el = ParserElement::new();
        let variable1_el = self.output[self.output.len() - 2].clone();
        let variable2_el = self.output[self.output.len() - 1].clone();
        //check both values have the same element type
        match (variable1_el.el_type, variable2_el.el_type) {
            (Some(el1_type), Some(el2_type)) => {
                if el1_type == el2_type {
                    match el1_type {
                        //if it's an el_int set the int64 of the new element to the sum of the 2 ints
                        ParserElementType::Int64 => {
                            match (variable1_el.int64, variable2_el.int64) {
                                (Some(val1), Some(val2)) => {
                                    el.int64 = Some(val1 + val2);
                                }
                                (_, _) => (),
                            }
                        }
                        ParserElementType::Str => {
                            match (variable1_el.string, variable2_el.string) {
                                (Some(_), Some(_)) => {
                                    self.success = false;
                                    original_self.display_error("fn_var_sum - can't sum strings");
                                    return self;
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
                    self = self.chomp_clear();
                    self
                } else {
                    self
                }
            }
            (_, _) => self,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    //A string
    fn test_el_string() {
        let input_str = "\"1234\"";
        let result = Parser::new2(input_str, Parser::el_str);
        assert_eq!(result.input_original, input_str);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(ParserElementType::Str));
        assert_eq!(result.output[0].string, Some("1234".to_string()));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);
    }

    #[test]
    //Next
    fn test_prim_next() {
        //Fails
        let input_str = "";
        let result = Parser::new2(input_str, Parser::prim_next);
        assert_eq!(result.input_original, input_str);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, false);

        //character alpha
        let input_str = "abc";
        let result = Parser::new2(input_str, Parser::prim_next);
        assert_eq!(result.input_original, input_str);
        assert_eq!(result.input_remaining, "bc");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "a");
        assert_eq!(result.success, true);

        //character number
        let input_str = "1bc";
        let result = Parser::new2(input_str, Parser::prim_next);
        assert_eq!(result.input_original, input_str);
        assert_eq!(result.input_remaining, "bc");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "1");
        assert_eq!(result.success, true);

        //character special
        let input_str = "~bc";
        let result = Parser::new2(input_str, Parser::prim_next);
        assert_eq!(result.input_original, input_str);
        assert_eq!(result.input_remaining, "bc");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "~");
        assert_eq!(result.success, true);

        //character backslash
        let input_str = "\\bc";
        let result = Parser::new2(input_str, Parser::prim_next);
        assert_eq!(result.input_original, input_str);
        assert_eq!(result.input_remaining, "bc");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "\\");
        assert_eq!(result.success, true);

        //character unicode
        let input_str = "ébc";
        let result = Parser::new2(input_str, Parser::prim_next);
        assert_eq!(result.input_original, input_str);
        assert_eq!(result.input_remaining, "bc");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "é");
        assert_eq!(result.success, true);
    }

    //= x + 1 2
    //print x
    //= y + 1.1 2.2
    //print y
    #[test]
    fn test_run2() {
        let func = |p| Parser::combi_first_success_of(p, &[Parser::fn_var_assign].to_vec());
        let input_str = "= x 123";
        let result = Parser::new2(input_str, func);
        assert_eq!(result.input_original, input_str);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(ParserElementType::Var));
        assert_eq!(result.output[0].var_name, Some("x".to_string()));
        assert_eq!(result.output[0].int64, Some(123));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);
    }
    #[test]
    fn test_variable_sum() {
        //not a valid el_var sum
        let mut parser = Parser::new(" + test 1");
        parser.display_errors = false;
        let result = parser.clone().fn_var_sum();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, " + test 1");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, false);

        //short el_int plus short el_int, with combi_optional brackets
        parser = Parser::new("(+ 1 2)");
        parser.display_errors = false;
        let result = parser.clone().fn_var_sum();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(ParserElementType::Int64));
        assert_eq!(result.output[0].int64, Some(3));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //short el_int plus short el_int
        parser = Parser::new("+ 1 2");
        parser.display_errors = false;
        let result = parser.clone().fn_var_sum();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(ParserElementType::Int64));
        assert_eq!(result.output[0].int64, Some(3));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //long el_int plus long el_int
        parser = Parser::new("+ 11111 22222");
        parser.display_errors = false;
        let result = parser.clone().fn_var_sum();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(ParserElementType::Int64));
        assert_eq!(result.output[0].int64, Some(33333));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //long el_int plus negative long el_int
        parser = Parser::new("+ 11111 -22222");
        parser.display_errors = false;
        let result = parser.clone().fn_var_sum();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(ParserElementType::Int64));
        assert_eq!(result.output[0].int64, Some(-11111));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //short el_float plus short el_float
        parser = Parser::new("+ 1.1 2.2");
        parser.display_errors = false;
        let result = parser.clone().fn_var_sum();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(ParserElementType::Float64));
        assert_eq!(result.output[0].float64, Some(3.3000000000000003)); // yikes, floats
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //long el_float plus long el_float
        parser = Parser::new("+ 11111.11111 22222.22222");
        parser.display_errors = false;
        let result = parser.clone().fn_var_sum();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(ParserElementType::Float64));
        assert_eq!(result.output[0].float64, Some(33333.33333));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //long el_float plus negative long el_float
        parser = Parser::new("+ 11111.11111 -22222.22222");
        parser.display_errors = false;
        let result = parser.clone().fn_var_sum();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(ParserElementType::Float64));
        assert_eq!(result.output[0].float64, Some(-11111.11111));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);
    }
    #[test]
    fn test_multiple_variable_assign() {
        let mut parser = Parser::new("= x + 1 2\r\n= y + 3 4\r\n= z + 5.0 6.0");
        parser.display_errors = false;
        let result = parser.clone().parse();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 3);
        assert_eq!(result.output[0].el_type, Some(ParserElementType::Var));
        assert_eq!(result.output[0].var_name, Some("x".to_string()));
        assert_eq!(result.output[0].int64, Some(3));
        assert_eq!(result.output[1].el_type, Some(ParserElementType::Var));
        assert_eq!(result.output[1].var_name, Some("y".to_string()));
        assert_eq!(result.output[1].int64, Some(7));
        assert_eq!(result.output[2].el_type, Some(ParserElementType::Var));
        assert_eq!(result.output[2].var_name, Some("z".to_string()));
        assert_eq!(result.output[2].float64, Some(11.0));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //don't create new var_name if already exists, update it
        parser = Parser::new("= x + 1 2\r\n= x + 3 4");
        parser.display_errors = false;
        let result = parser.clone().parse();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(ParserElementType::Var));
        assert_eq!(result.output[0].var_name, Some("x".to_string()));
        assert_eq!(result.output[0].int64, Some(7));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);
    }
    #[test]
    fn test_variable_assign() {
        //not a el_var assignment
        let mut parser = Parser::new(" = x 1");
        parser.display_errors = false;
        let result = parser.clone().fn_var_assign();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, " = x 1");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, false);

        //"= x (+ 1 (+ 2 (+ 3 4)))", i.e. x = 1 + (2 + (3 + 4))
        //as below with brackets notation
        parser = Parser::new("= x (+ 1 (+ 2 (+ 3 4)))");
        parser.display_errors = false;
        let result = parser.clone().fn_var_assign();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(ParserElementType::Var));
        assert_eq!(result.output[0].var_name, Some("x".to_string()));
        assert_eq!(result.output[0].int64, Some(10));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //"= x + 1 + 2 + 3 4", i.e. x = 1 + (2 + (3 + 4))
        //short name el_var assignment to sum of 2 short ints, where the second is 2 nested sums of 2 short ints
        parser = Parser::new("= x + 1 + 2 + 3 4");
        parser.display_errors = false;
        let result = parser.clone().fn_var_assign();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(ParserElementType::Var));
        assert_eq!(result.output[0].var_name, Some("x".to_string()));
        assert_eq!(result.output[0].int64, Some(10));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //"= x + + 1 2 + 3 4", i.e. x = (1 + 2) + (3 + 4))
        //short name el_var assignment to sum of 2 short ints, where the second is 2 nested sums of 2 short ints, different format
        parser = Parser::new("= x + + 1 2 + 3 4");
        parser.display_errors = false;
        let result = parser.clone().fn_var_assign();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(ParserElementType::Var));
        assert_eq!(result.output[0].var_name, Some("x".to_string()));
        assert_eq!(result.output[0].int64, Some(10));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //"= x + 1 + 2 3", i.e. x = 1 + (2 + 3)
        //short name el_var assignment to sum of 2 short ints, where the second is a sum of 2 short ints
        parser = Parser::new("= x + 1 + 2 3");
        parser.display_errors = false;
        let result = parser.clone().fn_var_assign();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(ParserElementType::Var));
        assert_eq!(result.output[0].var_name, Some("x".to_string()));
        assert_eq!(result.output[0].int64, Some(6));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //short name el_var assignment to sum of 2 short ints
        parser = Parser::new("= x + 1 2");
        parser.display_errors = false;
        let result = parser.clone().fn_var_assign();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(ParserElementType::Var));
        assert_eq!(result.output[0].var_name, Some("x".to_string()));
        assert_eq!(result.output[0].int64, Some(3));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //short name el_var assignment to sum of 2 long floats
        parser = Parser::new("= x + 11111.11111 22222.22222");
        parser.display_errors = false;
        let result = parser.clone().fn_var_assign();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(ParserElementType::Var));
        assert_eq!(result.output[0].var_name, Some("x".to_string()));
        assert_eq!(result.output[0].float64, Some(33333.33333));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //short name el_var assignment to short el_int
        parser = Parser::new("= x 1");
        parser.display_errors = false;
        let result = parser.clone().fn_var_assign();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(ParserElementType::Var));
        assert_eq!(result.output[0].var_name, Some("x".to_string()));
        assert_eq!(result.output[0].int64, Some(1));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //short name el_var assignment to short el_int with newlines
        parser = Parser::new("= x 1\r\n\r\n\r\n");
        parser.display_errors = false;
        let result = parser.clone().fn_var_assign();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(ParserElementType::Var));
        assert_eq!(result.output[0].var_name, Some("x".to_string()));
        assert_eq!(result.output[0].int64, Some(1));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //long name el_var with grapheme assignment to long negative el_int
        parser = Parser::new("= éxample_long_variable_name -123456");
        parser.display_errors = false;
        let result = parser.clone().fn_var_assign();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(ParserElementType::Var));
        assert_eq!(
            result.output[0].var_name,
            Some("éxample_long_variable_name".to_string())
        );
        assert_eq!(result.output[0].int64, Some(-123456));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //short name el_var assignment to short el_float
        parser = Parser::new("= x 1.2");
        parser.display_errors = false;
        let result = parser.clone().fn_var_assign();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(ParserElementType::Var));
        assert_eq!(result.output[0].var_name, Some("x".to_string()));
        assert_eq!(result.output[0].float64, Some(1.2));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //short name el_var assignment to long negative el_float
        parser = Parser::new("= x 1.2");
        parser.display_errors = false;
        let result = parser.clone().fn_var_assign();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(ParserElementType::Var));
        assert_eq!(result.output[0].var_name, Some("x".to_string()));
        assert_eq!(result.output[0].float64, Some(1.2));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);
    }

    #[test]
    fn test_variable() {
        //not a el_var
        let mut parser = Parser::new(" x = 1");
        parser.display_errors = false;
        let result = parser.clone().el_var();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, " x = 1");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, false);

        //short name el_var
        parser = Parser::new("x = 1");
        parser.display_errors = false;
        let result = parser.clone().el_var();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "= 1");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(ParserElementType::Var));
        assert_eq!(result.output[0].var_name, Some("x".to_string()));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //long name el_var with grapheme
        parser = Parser::new("éxample_long_variable_name = 123.45");
        parser.display_errors = false;
        let result = parser.clone().el_var();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "= 123.45");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(ParserElementType::Var));
        assert_eq!(
            result.output[0].var_name,
            Some("éxample_long_variable_name".to_string())
        );
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);
    }
    #[test]
    fn test_float() {
        //not a el_float
        let mut parser = Parser::new("a123.456");
        parser.display_errors = false;
        let result = parser.clone().el_float();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "a123.456");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, false);

        //positive small el_float
        parser = Parser::new("12.34");
        parser.display_errors = false;
        let result = parser.clone().el_float();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(ParserElementType::Float64));
        assert_eq!(result.output[0].float64, Some(12.34));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //positive large el_float
        parser = Parser::new("123456.78");
        parser.display_errors = false;
        let result = parser.clone().el_float();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(ParserElementType::Float64));
        assert_eq!(result.output[0].float64, Some(123456.78));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //negative el_float
        parser = Parser::new("-123456.78");
        parser.display_errors = false;
        let result = parser.clone().el_float();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(ParserElementType::Float64));
        assert_eq!(result.output[0].float64, Some(-123456.78));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);
    }

    #[test]
    fn test_int() {
        //not an el_int
        let mut parser = Parser::new("a123");
        parser.display_errors = false;
        let result = parser.clone().el_int();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "a123");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, false);

        //positive small el_int
        parser = Parser::new("12");
        parser.display_errors = false;
        let result = parser.clone().el_int();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(ParserElementType::Int64));
        assert_eq!(result.output[0].int64, Some(12));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //positive large el_int
        parser = Parser::new("123456");
        parser.display_errors = false;
        let result = parser.clone().el_int();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(ParserElementType::Int64));
        assert_eq!(result.output[0].int64, Some(123456));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //negative el_int
        parser = Parser::new("-123456");
        parser.display_errors = false;
        let result = parser.clone().el_int();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0].el_type, Some(ParserElementType::Int64));
        assert_eq!(result.output[0].int64, Some(-123456));
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);
    }

    #[test]
    fn test_combi_optional() {
        let mut parser = Parser::new("a123Test");
        parser.display_errors = false;
        let result = parser.clone().combi_optional(Parser::prim_char);
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "123Test");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "a");
        assert_eq!(result.success, true);

        parser = Parser::new("a123Test");
        parser.display_errors = false;
        let result = parser.clone().combi_zero_or_more_of(Parser::prim_digit);
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "a123Test");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);
    }

    #[test]
    fn test_combi_zero_or_more_of() {
        let mut parser = Parser::new("a123Test");
        parser.display_errors = false;
        let result = parser.clone().combi_zero_or_more_of(Parser::prim_digit);
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "a123Test");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        parser = Parser::new("123Test");
        parser.display_errors = false;
        let result = parser.clone().combi_zero_or_more_of(Parser::prim_digit);
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "Test");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "123");
        assert_eq!(result.success, true);
    }
    #[test]
    fn test_combi_one_or_more_of() {
        let mut parser = Parser::new("a123Test");
        parser.display_errors = false;
        let result = parser.clone().combi_one_or_more_of(Parser::prim_digit);
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "a123Test");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, false);

        parser = Parser::new("123Test");
        parser.display_errors = false;
        let result = parser.clone().combi_one_or_more_of(Parser::prim_digit);
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "Test");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "123");
        assert_eq!(result.success, true);
    }

    #[test]
    fn test_multiple_parsers() {
        let mut parser = Parser::new("1Test");
        parser.display_errors = false;
        let result = parser.clone().prim_digit().prim_word("Te");
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "st");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "1Te");
        assert_eq!(result.success, true);
    }
    #[test]
    fn test_prim_eof_or_eol() {
        //not eof or eol
        let mut parser = Parser::new("1");
        parser.display_errors = false;
        let result = parser.clone().prim_eols_or_eof();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "1");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, false);

        //eof
        let mut parser = Parser::new("");
        parser.display_errors = false;
        let result = parser.clone().prim_eols_or_eof();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //single eol1
        let mut parser = Parser::new("\n");
        parser.display_errors = false;
        let result = parser.clone().prim_eols_or_eof();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "\n");
        assert_eq!(result.success, true);

        //single eol2
        let mut parser = Parser::new("\r\n");
        parser.display_errors = false;
        let result = parser.clone().prim_eols_or_eof();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "\r\n");
        assert_eq!(result.success, true);

        //multiple eol1
        let mut parser = Parser::new("\n\n\n\n");
        parser.display_errors = false;
        let result = parser.clone().prim_eols_or_eof();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "\n\n\n\n");
        assert_eq!(result.success, true);

        //multiple eol2
        let mut parser = Parser::new("\r\n\r\n\r\n\r\n");
        parser.display_errors = false;
        let result = parser.clone().prim_eols_or_eof();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "\r\n\r\n\r\n\r\n");
        assert_eq!(result.success, true);
    }

    #[test]
    fn test_prim_eof() {
        //not eof
        let mut parser = Parser::new("1");
        parser.display_errors = false;
        let result = parser.clone().prim_eof();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "1");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, false);

        //eof
        let mut parser = Parser::new("");
        parser.display_errors = false;
        let result = parser.clone().prim_eof();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);
    }

    #[test]
    fn test_prim_eol() {
        //not an eol
        let mut parser = Parser::new("1");
        parser.display_errors = false;
        let result = parser.clone().prim_eol();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "1");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, false);

        //single eol1
        let mut parser = Parser::new("\n");
        parser.display_errors = false;
        let result = parser.clone().prim_eol();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "\n");
        assert_eq!(result.success, true);

        //single eol2
        let mut parser = Parser::new("\r\n");
        parser.display_errors = false;
        let result = parser.clone().prim_eol();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "\r\n");
        assert_eq!(result.success, true);

        //multiple eol1
        let mut parser = Parser::new("\n\n\n\n");
        parser.display_errors = false;
        let result = parser.clone().prim_eol();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "\n\n\n\n");
        assert_eq!(result.success, true);

        //multiple eol2
        let mut parser = Parser::new("\r\n\r\n\r\n\r\n");
        parser.display_errors = false;
        let result = parser.clone().prim_eol();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "\r\n\r\n\r\n\r\n");
        assert_eq!(result.success, true);
    }

    #[test]
    fn test_prim_digit() {
        let mut parser = Parser::new("123Test");
        parser.display_errors = false;
        let result = parser.clone().prim_digit().prim_digit().prim_digit();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "Test");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "123");
        assert_eq!(result.success, true);
    }
    #[test]
    fn test_prim_char() {
        //fail
        let mut parser = Parser::new("Te sting 123");
        parser.display_errors = false;
        let result = parser
            .clone()
            .prim_char()
            .prim_char()
            .prim_char()
            .prim_char();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, " sting 123");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "Te");
        assert_eq!(result.success, false);

        //succeed
        let mut parser = Parser::new("Testing 123");
        parser.display_errors = false;
        let result = parser
            .clone()
            .prim_char()
            .prim_char()
            .prim_char()
            .prim_char();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "ing 123");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "Test");
        assert_eq!(result.success, true);
    }

    #[test]
    fn test_prim_word() {
        let parser = Parser::new("Testing 123");
        let result = parser
            .clone()
            .prim_word("Test")
            .prim_word("ing")
            .prim_word(" ")
            .prim_word("123");
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output.len(), 0);
        assert_eq!(result.chomp, "Testing 123");
        assert_eq!(result.success, true);
    }
}
