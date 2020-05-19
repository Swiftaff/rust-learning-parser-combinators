use colored::*;
//extern crate derive_more;
//use derive_more::{Add, Display, From, Into};
use indextree;
use std::cmp::PartialEq;
use std::fmt;
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
| ,      | [prim_eols](#method.prim_eols)                 | ` `                  | `,`          | `(eols)`                   | true (in Parser.success) |
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
///- output_arena: is a list of ParserElements (in an 'indextree' Arena) generated by your parser functions
///- output_arena_node_parent_id: refers to the current parent node in the indextree Arena
///- chomp: is the sub-string built up by a subgroup of the current parser functions.<br />
///  It can be cleared manually with [chomp_clear](#method.chomp_clear) and is usually used to build some fragment of a string for e.g. a variable name
///- success: is set to true or false by the current parser function. Currently, if a fail occurs, it is passed through all functions until the last one<br />
///  (TODO) use Results, and Panic during main parser functions
#[derive(Debug, Clone)]
pub struct Parser {
    input_original: String,
    input_remaining: String,
    language_arena: indextree::Arena<ParserFunctionTypeAndParam>,
    language_arena_node_parent_id: indextree::NodeId,
    output_arena: indextree::Arena<ParserElement>,
    output_arena_node_parent_id: indextree::NodeId,
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
#[derive(Debug, Clone, PartialEq)]
pub enum ParserElementType {
    Int64,
    Float64,
    Var,
    Str,
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

//TODO tryout this simpler parser element
#[derive(Debug, Clone)]
pub struct ParserEl {
    el_type: Option<ParserElementType>,
    value: Option<ParserElValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParserElValue {
    I64(i64),
    F64(f64),
    Str(String),
    Var(String),
}

impl ParserEl {
    pub fn new() -> ParserEl {
        ParserEl {
            el_type: None,
            value: None,
        }
    }
}

#[derive(Clone)]
pub enum ParserFunctionType {
    None, //added while creating language_arena - might cause issue if not in match statements?
    TakesParser(ParserFunction), //e.g. primitive except prim_word, element, function
    TakesParserWord(ParserFunctionString), //e.g. prim_word
    TakesParserFn(ParserFunctionParserFunction), //e.g. simple combinator like combi_parser_one_or_more
                                                 //TakesParserVecFn(ParserFunction, ParserFunctionParam::Avec(<Vec<ParserFunction>>)), //e.g. combi_until_first_do_second
                                                 //TakesParserBVecFn(ParserFunction, Vec<ParserFunction>), //e.g. combi_until_first_do_second
}

///None, String, Parser, VecParser
#[derive(Clone)]
pub enum ParserFunctionParam {
    None,
    String(String),
    ParserFn(ParserFunction),
    VecParserFn(Vec<ParserFunction>),
}

pub type ParserFunction = fn(Parser) -> Parser;
pub type ParserFunctionString = fn(Parser, &str) -> Parser;
pub type ParserFunctionParserFunction = fn(Parser, ParserFunction) -> Parser;
pub type ParserFunctionTypeAndParam = (ParserFunctionType, ParserFunctionParam);

///quick and dirty helper function to Debug function names
//https://users.rust-lang.org/t/get-the-name-of-the-function-a-function-pointer-points-to/14930
fn get_parserfn_name(f: fn(Parser) -> Parser) -> &'static str {
    match f {
        _ if f == Parser::prim_next => "prim_next",
        _ => "unknown function name - manually add it to 'get_parserfn_name' to see it here!",
    }
}

impl fmt::Debug for ParserFunctionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParserFunctionType::None => write!(f, "None"),
            ParserFunctionType::TakesParser(p) => {
                write!(f, "TakesParser {:?}", get_parserfn_name(p))
            }
            ParserFunctionType::TakesParserWord(_) => write!(f, "TakesParserWord"),
            ParserFunctionType::TakesParserFn(_) => write!(f, "TakesParserFn"),
        }
    }
}

impl fmt::Debug for ParserFunctionParam {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParserFunctionParam::None => write!(f, "None"),
            ParserFunctionParam::String(_) => write!(f, "String"),
            ParserFunctionParam::ParserFn(_) => write!(f, "ParserFn"),
            ParserFunctionParam::VecParserFn(_) => write!(f, "VecParserFn"),
        }
    }
}

/// ## Main Methods
impl Parser {
    ///Initialises a new parser with the string you wish to parse
    pub fn new(input_string: &str) -> Parser {
        let mut output_arena: indextree::Arena<ParserElement> = indextree::Arena::new();
        let output_arena_root: ParserElement = ParserElement::new();
        let output_arena_node_parent_id = output_arena.new_node(output_arena_root);

        let mut language_arena: indextree::Arena<ParserFunctionTypeAndParam> =
            indextree::Arena::new();
        let language_root: ParserFunctionTypeAndParam =
            (ParserFunctionType::None, ParserFunctionParam::None);
        let language_arena_node_parent_id = language_arena.new_node(language_root);

        let new_parser = Parser {
            input_original: input_string.to_string(),
            input_remaining: input_string.to_string(),
            chomp: "".to_string(),
            chomping: true,
            language_arena,
            language_arena_node_parent_id,
            output_arena,
            output_arena_node_parent_id,
            success: true,
            display_errors: true,
        };
        new_parser
    }

    ///Defines the parser to run, then runs it on the initialised parser from new
    ///for now it only contains a few things...
    ///'fn_var_assign' which itself calls sub-parsers like el_int, el_float, fn_var_sum
    ///'prim_eols' to allow separating the variable assignments
    pub fn parse(mut self: Parser) -> Parser {
        while self.success && self.input_remaining.len() > 0 {
            self =
                self.combi_first_success_of(&[Parser::fn_var_assign, Parser::prim_eols].to_vec());
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
    ///let parse_result = rust_learning_parser_combinators::Parser::new_and_parse(string_to_parse, my_parser);
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
    ///let parse_result = rust_learning_parser_combinators::Parser::new_and_parse(string_to_parse, my_parser);
    ///println!("{:?}",parse_result);
    ///```

    pub fn new_and_parse<F>(input_string: &str, func: F) -> Parser
    where
        F: Fn(Parser) -> Parser,
    {
        let new_parser = Parser::new(input_string);
        func(new_parser)
    }

    pub fn new_and_parse_aliases(input_string: &str, parser_lang_string: &str) -> Parser {
        //first, parse the parser_lang_string to get the series of your parser instructions
        let mut parser_lang: Parser = Parser::new(parser_lang_string);
        while parser_lang.success && parser_lang.input_remaining.len() > 0 {
            parser_lang = parser_lang.lang_one_of_all_lang_parsers();
            parser_lang
                .clone()
                .test_printing_functionTypeAndParam("last child added: ");
        }

        //second, parse the input_string using those instructions
        let mut parser: Parser = Parser::new(input_string);
        let language_arena = &mut parser_lang.clone().language_arena;
        //let language_arena_current_parent_node_id =
        //    parser_lang.clone().language_arena_node_parent_id;
        let list_of_nodes: Vec<&indextree::Node<ParserFunctionTypeAndParam>> = language_arena
            .iter()
            //exclude removed items
            .filter(|n| !n.is_removed())
            //exclude root item
            .filter(|n| {
                let (f, p) = n.get();
                match (f, p) {
                    (ParserFunctionType::None, ParserFunctionParam::None) => false,
                    _ => true,
                }
            })
            .collect();

        for node in list_of_nodes.clone() {
            let (f, param_option) = node.get();
            println!(
                "{:?} {:?} {:?}",
                list_of_nodes.clone().len(),
                f,
                param_option
            );
            match f {
                ParserFunctionType::TakesParser(fun) => {
                    parser = fun(parser);
                }
                ParserFunctionType::TakesParserWord(fun) => match param_option {
                    ParserFunctionParam::String(string) => {
                        parser = fun(parser, string.as_str());
                    }
                    _ => (),
                },
                _ => (),
            }
        }

        parser
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

    pub fn get_parser_function_by_name(name: String) -> ParserFunction {
        match name.as_str() {
            ">" => Parser::lang_prim_next,
            "\"" => Parser::lang_prim_quote,
            //TODO handle prim_word
            "@" => Parser::lang_prim_char,
            "#" => Parser::lang_prim_digit,
            "," => Parser::lang_prim_eols,
            "." => Parser::lang_prim_eof,
            ";" => Parser::lang_prim_eols_or_eof,
            //TODO handle combinators
            //TODO handle elements
            _ => Parser::lang_prim_eof,
        }
    }
}

///### Arena Helpers
///wrapping basic functions for indextree
impl Parser {
    ///Finds a variable by name if the parser created it already<br/>
    ///Option...<br/>
    ///Some(index of the found variable within the parser.output, and the variable[ParserElement](struct.ParserElement.html)<br/>
    ///None
    pub fn output_arena_find_element_var(
        mut self: Parser,
        var_name: &str,
    ) -> Option<ParserElement> {
        let arena = &mut self.output_arena;
        //there should only be one or zero
        let list_of_nodes_with_var_name: Vec<&indextree::Node<ParserElement>> = arena
            .iter()
            .filter(|x| x.get().var_name == Some(var_name.to_string()))
            .collect();
        if list_of_nodes_with_var_name.len() > 0 {
            Some(list_of_nodes_with_var_name[0].get().clone())
        } else {
            None
        }
    }

    pub fn output_arena_append_element(mut self: Parser, el: ParserElement) -> Parser {
        let arena = &mut self.output_arena;
        let new_node = arena.new_node(el);
        self.output_arena_node_parent_id.append(new_node, arena);
        self
    }

    pub fn language_arena_append_functionTypeAndParam(
        mut self: Parser,
        fp: ParserFunctionTypeAndParam,
    ) -> Parser {
        let arena = &mut self.language_arena;
        let new_node = arena.new_node(fp);
        self.language_arena_node_parent_id.append(new_node, arena);
        self
    }

    pub fn test_printing_functionTypeAndParam(mut self: Parser, s: &str) {
        let test_language_arena = &mut self.language_arena;
        let last_child = self
            .clone()
            .language_arena_get_last_child_functionTypeAndParam();
        println!("####{} {:?}", s, last_child);
    }

    pub fn output_arena_get_current_parent_element(mut self: Parser) -> Option<ParserElement> {
        let arena = &mut self.output_arena;
        let output_arena_current_parent_node_id = self.output_arena_node_parent_id;
        let parent_option = arena.get(output_arena_current_parent_node_id);
        match parent_option {
            Some(parent) => Some(parent.get().clone()),
            _ => None,
        }
        //parent_option.get()
    }

    pub fn output_arena_get_last_child_element(mut self: Parser) -> Option<ParserElement> {
        let arena = &mut self.output_arena;
        let output_arena_current_parent_node_id = self.output_arena_node_parent_id;
        //get parent node
        let parent_option = arena.get(output_arena_current_parent_node_id);
        match parent_option {
            Some(parent) => {
                //get last child id
                let last_child_id_option = parent.last_child();
                match last_child_id_option {
                    Some(last_child_id) => {
                        //get last child node
                        let child_option = arena.get(last_child_id);
                        match child_option {
                            Some(child) => Some(child.get().clone()),
                            _ => None,
                        }
                    }
                    _ => None,
                }
            }
            _ => None,
        }
        //parent_option.get()
    }

    pub fn language_arena_get_last_child_functionTypeAndParam(
        mut self: Parser,
    ) -> Option<ParserFunctionTypeAndParam> {
        let arena = &mut self.language_arena;
        let language_arena_current_parent_node_id = self.language_arena_node_parent_id;
        //get parent node
        let parent_option = arena.get(language_arena_current_parent_node_id);
        match parent_option {
            Some(parent) => {
                //get last child id
                let last_child_id_option = parent.last_child();
                match last_child_id_option {
                    Some(last_child_id) => {
                        //get last child node
                        let child_option = arena.get(last_child_id);
                        match child_option {
                            Some(child) => Some(child.get().clone()),
                            _ => None,
                        }
                    }
                    _ => None,
                }
            }
            _ => None,
        }
        //parent_option.get()
    }

    pub fn output_arena_get_nth_last_child_element(
        mut self: Parser,
        index: usize,
    ) -> Option<ParserElement> {
        let arena = &mut self.output_arena;
        let output_arena_current_parent_node_id = self.output_arena_node_parent_id;

        //get node_id
        let node_id_option = output_arena_current_parent_node_id
            .reverse_children(arena)
            .nth(index);
        match node_id_option {
            Some(node_id) => {
                //get node
                let node_option = arena.get(node_id);
                match node_option {
                    Some(node) => Some(node.get().clone()),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    pub fn output_arena_remove_nth_last_child_element(mut self: Parser, index: usize) -> Parser {
        let arena = &mut self.output_arena;
        let output_arena_current_parent_node_id = self.output_arena_node_parent_id;

        //get node_id
        let node_id_option = output_arena_current_parent_node_id
            .reverse_children(arena)
            .nth(index);
        match node_id_option {
            Some(node_id) => {
                //remove node
                node_id.remove(arena);
                self.output_arena = arena.clone();
                self
            }
            _ => self,
        }
    }
}
/// ## Language Aliases
///Functions to help decode a string of aliases of the parser functions of this module
impl Parser {
    pub fn lang_one_of_all_lang_parsers(self: Parser) -> Parser {
        self.combi_first_success_of(
            &[
                //combinators
                //Parser::lang_combi_one_or_more,
                //primitives
                Parser::lang_prim_word,
                Parser::lang_prim_eols_or_eof,
                Parser::lang_prim_eof,
                Parser::lang_prim_eols,
                Parser::lang_prim_quote,
                Parser::lang_prim_digit,
                Parser::lang_prim_char,
                Parser::lang_prim_next,
            ]
            .to_vec(),
        )
    }

    pub fn lang_factory_takes_parser(
        mut self: Parser,
        word: &str,
        pf: ParserFunctionTypeAndParam,
        error_text: &str,
    ) -> Parser {
        if self.success {
            self = self.prim_word(word);
            if self.success {
                self = self.language_arena_append_functionTypeAndParam(pf);
                self
            } else {
                self.display_error(error_text);
                self
            }
        } else {
            self
        }
    }

    //Primitives

    pub fn lang_prim_next(self: Parser) -> Parser {
        Parser::lang_factory_takes_parser(
            self,
            ">",
            (
                ParserFunctionType::TakesParser(Parser::prim_next),
                ParserFunctionParam::None,
            ),
            "lang_prim_next",
        )
    }

    pub fn lang_prim_quote(self: Parser) -> Parser {
        Parser::lang_factory_takes_parser(
            self,
            "\"",
            (
                ParserFunctionType::TakesParser(Parser::prim_quote),
                ParserFunctionParam::None,
            ),
            "lang_prim_quote",
        )
    }

    pub fn lang_prim_char(self: Parser) -> Parser {
        Parser::lang_factory_takes_parser(
            self,
            "@",
            (
                ParserFunctionType::TakesParser(Parser::prim_char),
                ParserFunctionParam::None,
            ),
            "lang_prim_char",
        )
    }

    pub fn lang_prim_digit(self: Parser) -> Parser {
        Parser::lang_factory_takes_parser(
            self,
            "#",
            (
                ParserFunctionType::TakesParser(Parser::prim_digit),
                ParserFunctionParam::None,
            ),
            "lang_prim_digit",
        )
    }

    pub fn lang_prim_eols(self: Parser) -> Parser {
        Parser::lang_factory_takes_parser(
            self,
            ",",
            (
                ParserFunctionType::TakesParser(Parser::prim_eols),
                ParserFunctionParam::None,
            ),
            "lang_prim_eols",
        )
    }

    pub fn lang_prim_eof(self: Parser) -> Parser {
        Parser::lang_factory_takes_parser(
            self,
            ".",
            (
                ParserFunctionType::TakesParser(Parser::prim_eof),
                ParserFunctionParam::None,
            ),
            "lang_prim_eof",
        )
    }

    pub fn lang_prim_eols_or_eof(self: Parser) -> Parser {
        Parser::lang_factory_takes_parser(
            self,
            ";",
            (
                ParserFunctionType::TakesParser(Parser::prim_eols_or_eof),
                ParserFunctionParam::None,
            ),
            "lang_prim_eols_or_eof",
        )
    }

    pub fn lang_prim_word(mut self: Parser) -> Parser {
        if self.success {
            let display_errors_previous_flag_setting = self.display_errors;
            self.display_errors = false;
            self = self.prim_quote_single().combi_until_first_do_second(
                [Parser::prim_quote_single, Parser::prim_next].to_vec(),
            );
            self.display_errors = display_errors_previous_flag_setting;
            if self.success {
                let fp = (
                    ParserFunctionType::TakesParserWord(Parser::prim_word),
                    ParserFunctionParam::String(self.clone().chomp),
                );
                self = self.language_arena_append_functionTypeAndParam(fp);
                self = self.chomp_clear();
                self
            } else {
                self.display_error("lang_prim_word");
                self
            }
        } else {
            self
        }
    }

    //Combinators

    pub fn lang_combi_one_or_more(mut self: Parser) -> Parser {
        if self.success {
            let display_errors_previous_flag_setting = self.display_errors;
            self.display_errors = false;
            self = self
                .prim_word("1+")
                .combi_zero_or_more_of(Parser::prim_space)
                .combi_until_first_do_second([Parser::prim_space, Parser::prim_next].to_vec());
            self.display_errors = display_errors_previous_flag_setting;
            if self.success {
                //TODO can't just do this because the chomp string, might be another nested combi...
                //ParserFunctionParam::ParserFn(Parser::get_parser_function_by_name(
                //    self.clone().chomp,
                //)),

                //Have to call the parser on that string
                //But it's getting a bit manual - need to look at a more nested AST approach?

                //TODO replace with output_arena_append_element
                /*self.language_arena.push((
                    ParserFunctionType::TakesParserFn(Parser::combi_one_or_more_of),
                    ParserFunctionParam::ParserFn(Parser::get_parser_function_by_name(
                        self.clone().chomp,
                    )),
                ));*/
                self = self.chomp_clear();
                self
            } else {
                self.display_error("lang_combi_one_or_more");
                self
            }
        } else {
            self
        }
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

    pub fn prim_space(mut self: Parser) -> Parser {
        let chomping_previous_flag_setting = self.chomping;
        self.chomping = false;
        self = self.prim_word(" ");
        self.chomping = chomping_previous_flag_setting;
        self
    }

    pub fn prim_quote(mut self: Parser) -> Parser {
        let chomping_previous_flag_setting = self.chomping;
        self.chomping = false;
        self = self.prim_word("\"");
        self.chomping = chomping_previous_flag_setting;
        self
    }

    pub fn prim_quote_single(mut self: Parser) -> Parser {
        let chomping_previous_flag_setting = self.chomping;
        self.chomping = false;
        self = self.prim_word("'");
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
    pub fn prim_eols(mut self: Parser) -> Parser {
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
                self.display_error("prim_eols");
                self
            }
        } else {
            self.display_error("prim_eols");
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

    ///Matches either (prim_eolss)[#method.prim_eolss] or (prim_eof)[#method.prim_eof]
    pub fn prim_eols_or_eof(mut self: Parser) -> Parser {
        if self.success {
            let display_errors_previous_flag_setting = self.display_errors;
            self.display_errors = false;
            self = self.combi_first_success_of(&[Parser::prim_eols, Parser::prim_eof].to_vec());
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
                self = self.output_arena_append_element(el);
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
                self = self.output_arena_append_element(el);
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
                self = self.output_arena_append_element(el);
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
            self = self.output_arena_append_element(el);
            //println!("{:?}", el);
            self = self.chomp_clear();
            self.display_errors = display_errors_previous_flag_setting;
            self
        } else {
            self.display_error("el_var");
            self
        }
    }
}

/// ## Parser Functions
impl Parser {
    ///equals sign, el_var name, value (test using el_int for now), e.g. "= x 1" (x equals 1)
    pub fn fn_var_assign(self: Parser) -> Parser {
        let mut temp_self = self
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
            //get the previously parsed variable name, and variable value
            let variable_el_option = temp_self.clone().output_arena_get_nth_last_child_element(1);
            let value_el_option = temp_self.clone().output_arena_get_nth_last_child_element(0);
            //combine them into one element
            match (variable_el_option, value_el_option) {
                (Some(variable_el), Some(mut value_el)) => {
                    value_el.el_type = Some(ParserElementType::Var);
                    value_el.var_name = variable_el.var_name;
                    //remove those two last elements, and replace them with the combined element
                    temp_self = temp_self.output_arena_remove_nth_last_child_element(0);
                    temp_self = temp_self.output_arena_remove_nth_last_child_element(0);
                    //add combined element back into arena
                    temp_self = temp_self.output_arena_append_element(value_el);
                    temp_self = temp_self.chomp_clear();
                    temp_self
                }
                _ => {
                    temp_self.display_error("fn_var_assign - no variable or value found to assign");
                    temp_self
                }
            }
        } else {
            temp_self.display_error("fn_var_assign");
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
        //check both values exist
        let variable2_el_option = self.clone().output_arena_get_nth_last_child_element(0);
        let variable1_el_option = self.clone().output_arena_get_nth_last_child_element(1);
        match (variable1_el_option, variable2_el_option) {
            (Some(variable1_el), Some(variable2_el)) => {
                //check both values have the same element type
                match (variable1_el.el_type, variable2_el.el_type) {
                    (Some(el1_type), Some(el2_type)) => {
                        if el1_type == el2_type {
                            match el1_type {
                                //if it's an el_int set the int64 of the first element to be the sum of the 2 ints
                                //(because we will remove the second element)
                                ParserElementType::Int64 => {
                                    match (variable1_el.int64, variable2_el.int64) {
                                        (Some(val1), Some(val2)) => {
                                            el.el_type = Some(ParserElementType::Int64);
                                            el.int64 = Some(val1 + val2);
                                        }
                                        (_, _) => {
                                            original_self.success = false;
                                            original_self //original_self
                                                .display_error("fn_var_sum - can't find two Int64 values");
                                            return original_self;
                                        }
                                    }
                                }

                                //can't sum strings
                                ParserElementType::Str => {
                                    original_self.success = false;
                                    original_self //original_self
                                        .display_error("fn_var_sum - can't sum strings");
                                    return original_self;
                                }

                                //if it's an el_float set the Float64 of the first element to be the sum of the 2 floats
                                //(because we will remove the second element)
                                _ => {
                                    match (variable1_el.float64, variable2_el.float64) {
                                        (Some(val1), Some(val2)) => {
                                            el.el_type = Some(ParserElementType::Float64);
                                            el.float64 = Some(val1 + val2);
                                        }
                                        (_, _) => {
                                            original_self.success = false;
                                            original_self //original_self
                                                .display_error("fn_var_sum - can't find two Float64 values");
                                            return original_self;
                                        }
                                    }
                                }
                            }

                            //remove the last 2 value elements
                            self = self.output_arena_remove_nth_last_child_element(0);
                            self = self.output_arena_remove_nth_last_child_element(0);
                            //add combined (sum) element back into arena
                            self = self.output_arena_append_element(el);
                            self = self.chomp_clear();
                            self
                        } else {
                            self
                        }
                    }
                    (_, _) => self,
                }
            }
            _ => {
                original_self.display_error("fn_var_sum - can't find either or both values");
                original_self.success = false;
                original_self
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    ///TODO Need a way to test equlity of expected_result
    fn test_lang_one_of_all_lang_parsers() {
        let language_string = ">";
        let _expected_result = (
            ParserFunctionType::TakesParser(Parser::prim_next),
            ParserFunctionParam::None,
        );
        let p = Parser::new(language_string);
        let result = Parser::lang_one_of_all_lang_parsers(p);
        assert_eq!(result.input_original, language_string);
    }

    #[test]
    fn test_get_parser_function_by_name() {
        assert_eq!(
            Parser::get_parser_function_by_name(">".to_string()) == Parser::lang_prim_next,
            true
        );
    }
    //================================================================================
    //Start Language Aliases Testing
    //================================================================================

    //lang_combinators

    #[test]
    fn test_lang_combi_one_or_more() {
        //only combis of prims so far
        let input_str = "aaaa";
        let language_string = "1+a";
        let result = Parser::new_and_parse_aliases(input_str, language_string);
        assert_eq!(result.input_original, input_str);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.chomp, "aaaa");
        assert_eq!(result.success, true);
    }

    #[test]
    fn test_lang_prim_word() {
        let input_str = "test";
        let language_string = "'test'";
        let result = Parser::new_and_parse_aliases(input_str, language_string);
        assert_eq!(result.input_original, input_str);
        //assert_eq!(result.input_remaining, "");
        assert_eq!(result.chomp, "test");
        assert_eq!(result.success, true);
    }

    #[test]
    fn test_lang_prim_eols_or_eof() {
        let mut input_str = "\r\n\r\n\r\n!\n";
        let mut language_string = ",@,";
        let mut result = Parser::new_and_parse_aliases(input_str, language_string);
        assert_eq!(result.input_original, input_str);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.chomp, "\r\n\r\n\r\n!\n");
        assert_eq!(result.success, true);

        input_str = "a";
        language_string = "@.";
        result = Parser::new_and_parse_aliases(input_str, language_string);
        assert_eq!(result.input_original, input_str);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.chomp, "a");
        assert_eq!(result.success, true);
    }

    #[test]
    fn test_lang_prim_eof() {
        let input_str = "a";
        let language_string = "@.";
        let result = Parser::new_and_parse_aliases(input_str, language_string);
        assert_eq!(result.input_original, input_str);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.chomp, "a");
        assert_eq!(result.success, true);
    }

    #[test]
    fn test_lang_prim_eols() {
        let input_str = "\r\n\r\n\r\n!\n";
        let language_string = ",@,";
        let result = Parser::new_and_parse_aliases(input_str, language_string);
        assert_eq!(result.input_original, input_str);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.chomp, "\r\n\r\n\r\n!\n");
        assert_eq!(result.success, true);
    }

    #[test]
    fn test_lang_prim_digit() {
        let input_str = "0123456789";
        let language_string = "##########";
        let result = Parser::new_and_parse_aliases(input_str, language_string);
        assert_eq!(result.input_original, input_str);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.chomp, "0123456789");
        assert_eq!(result.success, true);
    }

    #[test]
    fn test_lang_prim_char() {
        let input_str = "+%!";
        let language_string = "@@@";
        let result = Parser::new_and_parse_aliases(input_str, language_string);
        assert_eq!(result.input_original, input_str);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.chomp, "+%!");
        assert_eq!(result.success, true);
    }

    #[test]
    fn test_lang_prim_quote() {
        let input_str = "\"\"\"";
        let language_string = "\"\"\"";
        let result = Parser::new_and_parse_aliases(input_str, language_string);
        assert_eq!(result.input_original, input_str);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);
    }

    #[test]
    fn test_lang_prim_next() {
        let input_str = "123";
        let language_string = ">>>";
        let result = Parser::new_and_parse_aliases(input_str, language_string);
        assert_eq!(result.input_original, input_str);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.chomp, "123");
        assert_eq!(result.success, true);
    }
    //================================================================================
    //End Language Aliases Testing
    //================================================================================

    #[test]
    //A string
    fn test_el_string() {
        let input_str = "\"1234\"";
        let result = Parser::new_and_parse(input_str, Parser::el_str);
        assert_eq!(result.input_original, input_str);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.output_arena.count(), 2);
        let el_option = result.clone().output_arena_get_last_child_element();
        match el_option {
            Some(el) => {
                assert_eq!(el.el_type, Some(ParserElementType::Str));
                assert_eq!(el.string, Some("1234".to_string()));
            }
            _ => assert!(true, false),
        }
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);
    }

    #[test]
    //Next
    fn test_prim_next() {
        //Fails
        let input_str = "";
        let result = Parser::new_and_parse(input_str, Parser::prim_next);
        assert_eq!(result.input_original, input_str);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, false);

        //character alpha
        let input_str = "abc";
        let result = Parser::new_and_parse(input_str, Parser::prim_next);
        assert_eq!(result.input_original, input_str);
        assert_eq!(result.input_remaining, "bc");
        assert_eq!(result.chomp, "a");
        assert_eq!(result.success, true);

        //character number
        let input_str = "1bc";
        let result = Parser::new_and_parse(input_str, Parser::prim_next);
        assert_eq!(result.input_original, input_str);
        assert_eq!(result.input_remaining, "bc");
        assert_eq!(result.chomp, "1");
        assert_eq!(result.success, true);

        //character special
        let input_str = "~bc";
        let result = Parser::new_and_parse(input_str, Parser::prim_next);
        assert_eq!(result.input_original, input_str);
        assert_eq!(result.input_remaining, "bc");
        assert_eq!(result.chomp, "~");
        assert_eq!(result.success, true);

        //character backslash
        let input_str = "\\bc";
        let result = Parser::new_and_parse(input_str, Parser::prim_next);
        assert_eq!(result.input_original, input_str);
        assert_eq!(result.input_remaining, "bc");
        assert_eq!(result.chomp, "\\");
        assert_eq!(result.success, true);

        //character unicode
        let input_str = "ébc";
        let result = Parser::new_and_parse(input_str, Parser::prim_next);
        assert_eq!(result.input_original, input_str);
        assert_eq!(result.input_remaining, "bc");
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
        let result = Parser::new_and_parse(input_str, func);
        assert_eq!(result.input_original, input_str);
        assert_eq!(result.input_remaining, "");
        assert_eq!(
            result
                .output_arena
                .iter()
                .filter(|n| !n.is_removed())
                .count(),
            2
        );
        let el_option = result.clone().output_arena_get_last_child_element();
        match el_option {
            Some(el) => {
                assert_eq!(el.el_type, Some(ParserElementType::Var));
                assert_eq!(el.var_name, Some("x".to_string()));
                assert_eq!(el.int64, Some(123));
            }
            _ => assert!(true, false),
        }
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
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, false);

        //short el_int plus short el_int, with combi_optional brackets
        parser = Parser::new("(+ 1 2)");
        parser.display_errors = false;
        let result = parser.clone().fn_var_sum();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        let el_option = result.clone().output_arena_get_last_child_element();
        match el_option {
            Some(el) => {
                assert_eq!(el.el_type, Some(ParserElementType::Int64));
                assert_eq!(el.int64, Some(3));
            }
            _ => assert!(true, false),
        }
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //short el_int plus short el_int
        parser = Parser::new("+ 1 2");
        parser.display_errors = false;
        let result = parser.clone().fn_var_sum();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        let el_option = result.clone().output_arena_get_last_child_element();
        match el_option {
            Some(el) => {
                assert_eq!(el.el_type, Some(ParserElementType::Int64));
                assert_eq!(el.int64, Some(3));
            }
            _ => assert!(true, false),
        }
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //long el_int plus long el_int
        parser = Parser::new("+ 11111 22222");
        parser.display_errors = false;
        let result = parser.clone().fn_var_sum();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        let el_option = result.clone().output_arena_get_last_child_element();
        match el_option {
            Some(el) => {
                assert_eq!(el.el_type, Some(ParserElementType::Int64));
                assert_eq!(el.int64, Some(33333));
            }
            _ => assert!(true, false),
        }
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //long el_int plus negative long el_int
        parser = Parser::new("+ 11111 -22222");
        parser.display_errors = false;
        let result = parser.clone().fn_var_sum();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        let el_option = result.clone().output_arena_get_last_child_element();
        match el_option {
            Some(el) => {
                assert_eq!(el.el_type, Some(ParserElementType::Int64));
                assert_eq!(el.int64, Some(-11111));
            }
            _ => assert!(true, false),
        }
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //short el_float plus short el_float
        parser = Parser::new("+ 1.1 2.2");
        parser.display_errors = false;
        let result = parser.clone().fn_var_sum();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        let el_option = result.clone().output_arena_get_last_child_element();
        match el_option {
            Some(el) => {
                assert_eq!(el.el_type, Some(ParserElementType::Float64));
                assert_eq!(el.float64, Some(3.3000000000000003)); // yikes, floats
            }
            _ => assert!(true, false),
        }
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //long el_float plus long el_float
        parser = Parser::new("+ 11111.11111 22222.22222");
        parser.display_errors = false;
        let result = parser.clone().fn_var_sum();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        let el_option = result.clone().output_arena_get_last_child_element();
        match el_option {
            Some(el) => {
                assert_eq!(el.el_type, Some(ParserElementType::Float64));
                assert_eq!(el.float64, Some(33333.33333)); // yikes, floats
            }
            _ => assert!(true, false),
        }
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //long el_float plus negative long el_float
        parser = Parser::new("+ 11111.11111 -22222.22222");
        parser.display_errors = false;
        let result = parser.clone().fn_var_sum();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        let el_option = result.clone().output_arena_get_last_child_element();
        match el_option {
            Some(el) => {
                assert_eq!(el.el_type, Some(ParserElementType::Float64));
                assert_eq!(el.float64, Some(-11111.11111)); // yikes, floats
            }
            _ => assert!(true, false),
        }
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);
    }
    #[test]
    fn test_multiple_variable_assign() {
        let input_string = "= x + 1 2\r\n= y + 3 4\r\n= z + 5.0 6.0";
        let mut parser = Parser::new(input_string);
        //parser.display_errors = false;
        let result = parser.parse();
        assert_eq!(result.input_original, input_string);
        assert_eq!(result.input_remaining, "");

        let mut el_option = result.clone().output_arena_get_nth_last_child_element(2);
        match el_option {
            Some(el) => {
                assert_eq!(el.el_type, Some(ParserElementType::Var));
                assert_eq!(el.var_name, Some("x".to_string()));
                assert_eq!(el.int64, Some(3));
            }
            _ => assert!(true, false),
        }

        el_option = result.clone().output_arena_get_nth_last_child_element(1);
        match el_option {
            Some(el) => {
                assert_eq!(el.el_type, Some(ParserElementType::Var));
                assert_eq!(el.var_name, Some("y".to_string()));
                assert_eq!(el.int64, Some(7));
            }
            _ => assert!(true, false),
        }

        el_option = result.clone().output_arena_get_nth_last_child_element(0);
        match el_option {
            Some(el) => {
                assert_eq!(el.el_type, Some(ParserElementType::Var));
                assert_eq!(el.var_name, Some("z".to_string()));
                assert_eq!(el.float64, Some(11.0));
            }
            _ => assert!(true, false),
        }
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //don't create new var_name if already exists, update it
        parser = Parser::new("= x + 1 2\r\n= x + 3 4");
        parser.display_errors = false;
        let result = parser.clone().parse();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        let el_option = result.clone().output_arena_get_last_child_element();
        match el_option {
            Some(el) => {
                assert_eq!(el.el_type, Some(ParserElementType::Var));
                assert_eq!(el.var_name, Some("x".to_string()));
                assert_eq!(el.int64, Some(7));
            }
            _ => assert!(true, false),
        }
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);
    }
    #[test]
    fn test_variable_assign() {
        //not a el_var assignment
        let mut input_string = " = x 1";
        let mut result = Parser::new_and_parse(input_string, Parser::fn_var_assign);
        assert_eq!(result.input_original, input_string);
        assert_eq!(result.input_remaining, " = x 1");
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, false);

        //"= x (+ 1 (+ 2 (+ 3 4)))", i.e. x = 1 + (2 + (3 + 4))
        //as below with brackets notation
        input_string = "= x (+ 1 (+ 2 (+ 3 4)))";
        result = Parser::new_and_parse(input_string, Parser::fn_var_assign);
        assert_eq!(result.input_original, input_string);
        assert_eq!(result.input_remaining, "");
        let el_option = result.clone().output_arena_get_last_child_element();
        match el_option {
            Some(el) => {
                assert_eq!(el.el_type, Some(ParserElementType::Var));
                assert_eq!(el.var_name, Some("x".to_string()));
                assert_eq!(el.int64, Some(10));
            }
            _ => assert!(true, false),
        }
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //"= x + 1 + 2 + 3 4", i.e. x = 1 + (2 + (3 + 4))
        //short name el_var assignment to sum of 2 short ints, where the second is 2 nested sums of 2 short ints
        input_string = "= x + 1 + 2 + 3 4";
        result = Parser::new_and_parse(input_string, Parser::fn_var_assign);
        assert_eq!(result.input_original, input_string);
        assert_eq!(result.input_remaining, "");
        let el_option = result.clone().output_arena_get_last_child_element();
        match el_option {
            Some(el) => {
                assert_eq!(el.el_type, Some(ParserElementType::Var));
                assert_eq!(el.var_name, Some("x".to_string()));
                assert_eq!(el.int64, Some(10));
            }
            _ => assert!(true, false),
        }
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //"= x + + 1 2 + 3 4", i.e. x = (1 + 2) + (3 + 4))
        //short name el_var assignment to sum of 2 short ints, where the second is 2 nested sums of 2 short ints, different format
        input_string = "= x + + 1 2 + 3 4";
        result = Parser::new_and_parse(input_string, Parser::fn_var_assign);
        assert_eq!(result.input_original, input_string);
        assert_eq!(result.input_remaining, "");
        let el_option = result.clone().output_arena_get_last_child_element();
        match el_option {
            Some(el) => {
                assert_eq!(el.el_type, Some(ParserElementType::Var));
                assert_eq!(el.var_name, Some("x".to_string()));
                assert_eq!(el.int64, Some(10));
            }
            _ => assert!(true, false),
        }
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //"= x + 1 + 2 3", i.e. x = 1 + (2 + 3)
        //short name el_var assignment to sum of 2 short ints, where the second is a sum of 2 short ints
        input_string = "= x + 1 + 2 3";
        result = Parser::new_and_parse(input_string, Parser::fn_var_assign);
        assert_eq!(result.input_original, input_string);
        assert_eq!(result.input_remaining, "");
        let el_option = result.clone().output_arena_get_last_child_element();
        match el_option {
            Some(el) => {
                assert_eq!(el.el_type, Some(ParserElementType::Var));
                assert_eq!(el.var_name, Some("x".to_string()));
                assert_eq!(el.int64, Some(6));
            }
            _ => assert!(true, false),
        }
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //short name el_var assignment to sum of 2 short ints
        input_string = "= x + 1 2";
        result = Parser::new_and_parse(input_string, Parser::fn_var_assign);
        assert_eq!(result.input_original, input_string);
        assert_eq!(result.input_remaining, "");
        let el_option = result.clone().output_arena_get_last_child_element();
        match el_option {
            Some(el) => {
                assert_eq!(el.el_type, Some(ParserElementType::Var));
                assert_eq!(el.var_name, Some("x".to_string()));
                assert_eq!(el.int64, Some(3));
            }
            _ => assert!(true, false),
        }
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //short name el_var assignment to sum of 2 long floats
        input_string = "= x + 11111.11111 22222.22222";
        result = Parser::new_and_parse(input_string, Parser::fn_var_assign);
        assert_eq!(result.input_original, input_string);
        assert_eq!(result.input_remaining, "");
        let el_option = result.clone().output_arena_get_last_child_element();
        match el_option {
            Some(el) => {
                assert_eq!(el.el_type, Some(ParserElementType::Var));
                assert_eq!(el.var_name, Some("x".to_string()));
                assert_eq!(el.float64, Some(33333.33333));
            }
            _ => assert!(true, false),
        }
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //short name el_var assignment to short el_int
        input_string = "= x 1";
        result = Parser::new_and_parse(input_string, Parser::fn_var_assign);
        assert_eq!(result.input_original, input_string);
        assert_eq!(result.input_remaining, "");
        let el_option = result.clone().output_arena_get_last_child_element();
        match el_option {
            Some(el) => {
                assert_eq!(el.el_type, Some(ParserElementType::Var));
                assert_eq!(el.var_name, Some("x".to_string()));
                assert_eq!(el.int64, Some(1));
            }
            _ => assert!(true, false),
        }
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //short name el_var assignment to short el_int with newlines
        input_string = "= x 1\r\n\r\n\r\n";
        result = Parser::new_and_parse(input_string, Parser::fn_var_assign);
        assert_eq!(result.input_original, input_string);
        assert_eq!(result.input_remaining, "");
        let el_option = result.clone().output_arena_get_last_child_element();
        match el_option {
            Some(el) => {
                assert_eq!(el.el_type, Some(ParserElementType::Var));
                assert_eq!(el.var_name, Some("x".to_string()));
                assert_eq!(el.int64, Some(1));
            }
            _ => assert!(true, false),
        }
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //long name el_var with grapheme assignment to long negative el_int
        input_string = "= éxample_long_variable_name -123456";
        result = Parser::new_and_parse(input_string, Parser::fn_var_assign);
        assert_eq!(result.input_original, input_string);
        assert_eq!(result.input_remaining, "");
        let el_option = result.clone().output_arena_get_last_child_element();
        match el_option {
            Some(el) => {
                assert_eq!(el.el_type, Some(ParserElementType::Var));
                assert_eq!(el.var_name, Some("éxample_long_variable_name".to_string()));
                assert_eq!(el.int64, Some(-123456));
            }
            _ => assert!(true, false),
        }
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //short name el_var assignment to short el_float
        input_string = "= x 1.2";
        result = Parser::new_and_parse(input_string, Parser::fn_var_assign);
        assert_eq!(result.input_original, input_string);
        assert_eq!(result.input_remaining, "");
        let el_option = result.clone().output_arena_get_last_child_element();
        match el_option {
            Some(el) => {
                assert_eq!(el.el_type, Some(ParserElementType::Var));
                assert_eq!(el.var_name, Some("x".to_string()));
                assert_eq!(el.float64, Some(1.2));
            }
            _ => assert!(true, false),
        }
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //short name el_var assignment to long negative el_float
        input_string = "= x -11111.22222";
        result = Parser::new_and_parse(input_string, Parser::fn_var_assign);
        assert_eq!(result.input_original, input_string);
        assert_eq!(result.input_remaining, "");
        let el_option = result.clone().output_arena_get_last_child_element();
        match el_option {
            Some(el) => {
                assert_eq!(el.el_type, Some(ParserElementType::Var));
                assert_eq!(el.var_name, Some("x".to_string()));
                assert_eq!(el.float64, Some(-11111.22222));
            }
            _ => assert!(true, false),
        }
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
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, false);

        //short name el_var
        parser = Parser::new("x = 1");
        parser.display_errors = false;
        let result = parser.clone().el_var();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "= 1");
        let el_option = result.clone().output_arena_get_last_child_element();
        match el_option {
            Some(el) => {
                assert_eq!(el.el_type, Some(ParserElementType::Var));
                assert_eq!(el.var_name, Some("x".to_string()));
            }
            _ => assert!(true, false),
        }
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //long name el_var with grapheme
        parser = Parser::new("éxample_long_variable_name = 123.45");
        parser.display_errors = false;
        let result = parser.clone().el_var();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "= 123.45");
        let el_option = result.clone().output_arena_get_last_child_element();
        match el_option {
            Some(el) => {
                assert_eq!(el.el_type, Some(ParserElementType::Var));
                assert_eq!(el.var_name, Some("éxample_long_variable_name".to_string()));
            }
            _ => assert!(true, false),
        }
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
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, false);

        //positive small el_float
        parser = Parser::new("12.34");
        parser.display_errors = false;
        let result = parser.clone().el_float();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        let el_option = result.clone().output_arena_get_last_child_element();
        match el_option {
            Some(el) => {
                assert_eq!(el.el_type, Some(ParserElementType::Float64));
                assert_eq!(el.float64, Some(12.34));
            }
            _ => assert!(true, false),
        }
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //positive large el_float
        parser = Parser::new("123456.78");
        parser.display_errors = false;
        let result = parser.clone().el_float();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        let el_option = result.clone().output_arena_get_last_child_element();
        match el_option {
            Some(el) => {
                assert_eq!(el.el_type, Some(ParserElementType::Float64));
                assert_eq!(el.float64, Some(123456.78));
            }
            _ => assert!(true, false),
        }
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //negative el_float
        parser = Parser::new("-123456.78");
        parser.display_errors = false;
        let result = parser.clone().el_float();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        let el_option = result.clone().output_arena_get_last_child_element();
        match el_option {
            Some(el) => {
                assert_eq!(el.el_type, Some(ParserElementType::Float64));
                assert_eq!(el.float64, Some(-123456.78));
            }
            _ => assert!(true, false),
        }
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
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, false);

        //positive small el_int
        parser = Parser::new("12");
        parser.display_errors = false;
        let result = parser.clone().el_int();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        let el_option = result.clone().output_arena_get_last_child_element();
        match el_option {
            Some(el) => {
                assert_eq!(el.el_type, Some(ParserElementType::Int64));
                assert_eq!(el.int64, Some(12));
            }
            _ => assert!(true, false),
        }
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //positive large el_int
        parser = Parser::new("123456");
        parser.display_errors = false;
        let result = parser.clone().el_int();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        let el_option = result.clone().output_arena_get_last_child_element();
        match el_option {
            Some(el) => {
                assert_eq!(el.el_type, Some(ParserElementType::Int64));
                assert_eq!(el.int64, Some(123456));
            }
            _ => assert!(true, false),
        }
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //negative el_int
        parser = Parser::new("-123456");
        parser.display_errors = false;
        let result = parser.clone().el_int();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        let el_option = result.clone().output_arena_get_last_child_element();
        match el_option {
            Some(el) => {
                assert_eq!(el.el_type, Some(ParserElementType::Int64));
                assert_eq!(el.int64, Some(-123456));
            }
            _ => assert!(true, false),
        }
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
        assert_eq!(result.chomp, "a");
        assert_eq!(result.success, true);

        parser = Parser::new("a123Test");
        parser.display_errors = false;
        let result = parser.clone().combi_zero_or_more_of(Parser::prim_digit);
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "a123Test");
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
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        parser = Parser::new("123Test");
        parser.display_errors = false;
        let result = parser.clone().combi_zero_or_more_of(Parser::prim_digit);
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "Test");
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
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, false);

        parser = Parser::new("123Test");
        parser.display_errors = false;
        let result = parser.clone().combi_one_or_more_of(Parser::prim_digit);
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "Test");
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
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, false);

        //eof
        let mut parser = Parser::new("");
        parser.display_errors = false;
        let result = parser.clone().prim_eols_or_eof();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);

        //single eol1
        let mut parser = Parser::new("\n");
        parser.display_errors = false;
        let result = parser.clone().prim_eols_or_eof();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.chomp, "\n");
        assert_eq!(result.success, true);

        //single eol2
        let mut parser = Parser::new("\r\n");
        parser.display_errors = false;
        let result = parser.clone().prim_eols_or_eof();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.chomp, "\r\n");
        assert_eq!(result.success, true);

        //multiple eol1
        let mut parser = Parser::new("\n\n\n\n");
        parser.display_errors = false;
        let result = parser.clone().prim_eols_or_eof();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.chomp, "\n\n\n\n");
        assert_eq!(result.success, true);

        //multiple eol2
        let mut parser = Parser::new("\r\n\r\n\r\n\r\n");
        parser.display_errors = false;
        let result = parser.clone().prim_eols_or_eof();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
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
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, false);

        //eof
        let mut parser = Parser::new("");
        parser.display_errors = false;
        let result = parser.clone().prim_eof();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, true);
    }

    #[test]
    fn test_prim_eols() {
        //not an eol
        let mut parser = Parser::new("1");
        parser.display_errors = false;
        let result = parser.clone().prim_eols();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "1");
        assert_eq!(result.chomp, "");
        assert_eq!(result.success, false);

        //single eol1
        let mut parser = Parser::new("\n");
        parser.display_errors = false;
        let result = parser.clone().prim_eols();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.chomp, "\n");
        assert_eq!(result.success, true);

        //single eol2
        let mut parser = Parser::new("\r\n");
        parser.display_errors = false;
        let result = parser.clone().prim_eols();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.chomp, "\r\n");
        assert_eq!(result.success, true);

        //multiple eol1
        let mut parser = Parser::new("\n\n\n\n");
        parser.display_errors = false;
        let result = parser.clone().prim_eols();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
        assert_eq!(result.chomp, "\n\n\n\n");
        assert_eq!(result.success, true);

        //multiple eol2
        let mut parser = Parser::new("\r\n\r\n\r\n\r\n");
        parser.display_errors = false;
        let result = parser.clone().prim_eols();
        assert_eq!(result.input_original, parser.input_original);
        assert_eq!(result.input_remaining, "");
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
        assert_eq!(result.chomp, "Testing 123");
        assert_eq!(result.success, true);
    }
}
