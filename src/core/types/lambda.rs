/* core/types/lambda.rs */

use std::cell::RefCell;
use std::rc::Rc;

use crate::core::environment::Environment;
use crate::core::eval::eval;
use crate::core::types::error::arity_error;
use crate::core::types::error::Result;
use crate::core::types::function::Function;
use crate::core::types::symbol::Symbol;
use crate::core::value::Value;

#[derive(Debug, Clone)]
pub struct Lambda {
    pub args: Vec<Symbol>,
    pub body: Vec<Value>,
    pub environment: Rc<RefCell<Environment>>,
}

// TODO: &rest
impl Function for Lambda {
    fn call(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != self.args.len() {
            return Err(arity_error(self.args.len(), args.len()));
        }

        let local_env = Environment::new_local_environment(self.environment.clone());
        let mut ast = Vec::<Value>::new();

        for (sym, val) in self.args.iter().zip(args) {
            local_env.borrow_mut().insert_to_current(sym.clone(), val)?;
        }

        let mut result = Value::Nil;
        for val in &self.body {
            ast.push(val.clone());
            result = eval(&local_env, &mut ast)?;
        }

        Ok(result)
    }
}
