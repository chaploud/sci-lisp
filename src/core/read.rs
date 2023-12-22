/* read.rs */

use pest::iterators::Pair;

use crate::core::parse::Rule;
use crate::core::value::Value;

pub fn read(ast: &mut Vec<Value>, pair: Pair<Rule>) -> Result<Value, String> {

    let value = match pair.as_rule() {
        Rule::scilisp => read(ast, pair.into_inner().next().unwrap()),
        Rule::nil => Value::as_nil(),
        Rule::bool => Value::as_bool(pair),
        Rule::i64 => Value::as_i64(pair),
        Rule::f64 => Value::as_f64(pair),
        Rule::symbol => Value::as_symbol(pair),
        Rule::keyword => Value::as_keyword(pair),
        Rule::regex => Value::as_regex(pair),
        Rule::string => Value::as_string(pair),
        _ => unreachable!(), // COMMENT, WHITESPACE, etc...
    };

    match value {
        Ok(value) => {
            ast.push(value);
            Ok(Value::Nil)
        },
        Err(err) => Err(err),
    }
}

