/* core/environment.rs */

use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::rc::Rc;

use nohash::BuildNoHashHasher;

use crate::core::builtin::functions::*;
use crate::core::builtin::r#macros::*;
use crate::core::types::error::Error;
use crate::core::types::error::Result;
use crate::core::types::symbol::Symbol;
use crate::core::value::Value;

type Lookup = HashMap<Symbol, Value, BuildNoHashHasher<u64>>;

#[derive(Debug, PartialEq, Clone)]
pub struct Environment {
    pub parent: Option<Rc<RefCell<Environment>>>,
    pub current: Rc<RefCell<Lookup>>,
}

impl Environment {
    pub fn new_root_environment() -> Rc<RefCell<Self>> {
        let result = Rc::new(RefCell::new(Self {
            parent: None,
            current: Rc::new(RefCell::new(HashMap::default())),
        }));

        insert_builtin_macros(&mut result.borrow_mut());
        insert_builtin_functions(&mut result.borrow_mut());

        result
    }

    pub fn new_local_environment(parent: Rc<RefCell<Environment>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            parent: Some(parent.clone()),
            current: Rc::new(RefCell::new(HashMap::default())),
        }))
    }

    pub fn get(&self, key: &Symbol) -> Result<Value> {
        if let Some(value) = self.current.borrow().get(key) {
            return Ok(value.clone());
        }
        if let Some(parent) = self.parent.clone() {
            return parent.borrow().get(key);
        }

        Err(Error::Name(key.to_string()))
    }

    pub fn get_key_value(&self, key: &Symbol) -> Result<(Symbol, Value)> {
        if let Some((k, v)) = self.current.borrow().get_key_value(key) {
            return Ok((k.clone(), v.clone()));
        }
        if let Some(parent) = self.parent.clone() {
            return parent.borrow().get_key_value(key);
        }

        Err(Error::Name(key.to_string()))
    }

    pub fn insert(&mut self, key: &Symbol, value: Value) -> Result<()> {
        match self.current.borrow_mut().entry(key.clone()) {
            Entry::Occupied(mut entry) => {
                if entry.key().meta.mutable {
                    if !key.meta.mutable {
                        // overwrite with const
                        return Err(Error::Immutable(format!(
                            "cannot overwrite '{}' with const",
                            key
                        )));
                    }
                    entry.insert(value);
                } else {
                    return Err(Error::Immutable(format!(
                        "cannot overwrite immutable binding '{}'",
                        key
                    )));
                }
            }
            Entry::Vacant(entry) => {
                entry.insert(value);
            }
        };
        Ok(())
    }

    pub fn set(&mut self, key: &Symbol, value: Value) -> Result<()> {
        match self.current.borrow_mut().entry(key.clone()) {
            Entry::Occupied(mut entry) => {
                if entry.key().meta.mutable {
                    entry.insert(value);
                } else {
                    return Err(Error::Immutable(format!(
                        "cannot overwrite immutable binding '{}'",
                        key
                    )));
                }
            }
            Entry::Vacant(_) => {
                if let Some(parent) = self.parent.clone() {
                    parent.borrow_mut().set(key, value)?;
                }
                return Err(Error::Name(key.to_string()));
            }
        }
        Ok(())
    }
}

fn insert_builtin_functions(env: &mut Environment) {
    let _ = env.insert(&SYMBOL_TYPE, Value::Function(Rc::new(RefCell::new(TypeFn))));
    let _ = env.insert(
        &SYMBOL_PRINT,
        Value::Function(Rc::new(RefCell::new(PrintFn))),
    );
    let _ = env.insert(&SYMBOL_ADD, Value::Function(Rc::new(RefCell::new(AddFn))));
    let _ = env.insert(&SYMBOL_SUB, Value::Function(Rc::new(RefCell::new(SubFn))));
    let _ = env.insert(&SYMBOL_MUL, Value::Function(Rc::new(RefCell::new(MulFn))));
    let _ = env.insert(&SYMBOL_DIV, Value::Function(Rc::new(RefCell::new(DivFn))));
    let _ = env.insert(
        &SYMBOL_FLOORDIV,
        Value::Function(Rc::new(RefCell::new(FloorDivFn))),
    );
    let _ = env.insert(&SYMBOL_REM, Value::Function(Rc::new(RefCell::new(RemFn))));
    let _ = env.insert(
        &SYMBOL_EQUAL,
        Value::Function(Rc::new(RefCell::new(EqualFn))),
    );
    let _ = env.insert(
        &SYMBOL_NOTEQUAL,
        Value::Function(Rc::new(RefCell::new(NotEqualFn))),
    );
    let _ = env.insert(&SYMBOL_IS, Value::Function(Rc::new(RefCell::new(IsFn))));
    let _ = env.insert(&SYMBOL_GE, Value::Function(Rc::new(RefCell::new(GeFn))));
    let _ = env.insert(&SYMBOL_GT, Value::Function(Rc::new(RefCell::new(GtFn))));
    let _ = env.insert(&SYMBOL_LE, Value::Function(Rc::new(RefCell::new(LeFn))));
    let _ = env.insert(&SYMBOL_LT, Value::Function(Rc::new(RefCell::new(LtFn))));
    let _ = env.insert(&SYMBOL_STR, Value::Function(Rc::new(RefCell::new(StrFn))));
    let _ = env.insert(&SYMBOL_I64, Value::Function(Rc::new(RefCell::new(I64Fn))));
    let _ = env.insert(&SYMBOL_F64, Value::Function(Rc::new(RefCell::new(F64Fn))));
    let _ = env.insert(
        &SYMBOL_FIRST,
        Value::Function(Rc::new(RefCell::new(FirstFn))),
    );
    let _ = env.insert(&SYMBOL_REST, Value::Function(Rc::new(RefCell::new(RestFn))));
    let _ = env.insert(
        &SYMBOL_RANGE,
        Value::Function(Rc::new(RefCell::new(RangeFn))),
    );
    let _ = env.insert(
        &SYMBOL_GENSYM,
        Value::Function(Rc::new(RefCell::new(GensymFn { id: 0 }))),
    );
}

fn insert_builtin_macros(env: &mut Environment) {
    let _ = env.insert(&SYMBOL_DEF, Value::Macro(Rc::new(RefCell::new(DefMacro))));
    let _ = env.insert(
        &SYMBOL_CONST,
        Value::Macro(Rc::new(RefCell::new(ConstMacro))),
    );
    let _ = env.insert(&SYMBOL_SET, Value::Macro(Rc::new(RefCell::new(SetMacro))));
    let _ = env.insert(&SYMBOL_LET, Value::Macro(Rc::new(RefCell::new(LetMacro))));
    let _ = env.insert(
        &SYMBOL_QUOTE,
        Value::Macro(Rc::new(RefCell::new(QuoteMacro))),
    );
    let _ = env.insert(
        &SYMBOL_SYNTAX_QUOTE,
        Value::Macro(Rc::new(RefCell::new(SyntaxQuoteMacro))),
    );
    let _ = env.insert(&SYMBOL_DO, Value::Macro(Rc::new(RefCell::new(DoMacro))));
    let _ = env.insert(&SYMBOL_IF, Value::Macro(Rc::new(RefCell::new(IfMacro))));
    let _ = env.insert(
        &SYMBOL_WHILE,
        Value::Macro(Rc::new(RefCell::new(WhileMacro))),
    );
    let _ = env.insert(
        &SYMBOL_SWITCH,
        Value::Macro(Rc::new(RefCell::new(SwitchMacro))),
    );
    let _ = env.insert(&SYMBOL_TIME, Value::Macro(Rc::new(RefCell::new(TimeMacro))));
    let _ = env.insert(&SYMBOL_DOC, Value::Macro(Rc::new(RefCell::new(DocMacro))));
    let _ = env.insert(&SYMBOL_FN, Value::Macro(Rc::new(RefCell::new(FnMacro))));
    let _ = env.insert(&SYMBOL_DEFN, Value::Macro(Rc::new(RefCell::new(DefnMacro))));
    let _ = env.insert(
        &SYMBOL_THREAD_FIRST,
        Value::Macro(Rc::new(RefCell::new(ThreadFirstMacro))),
    );
    let _ = env.insert(
        &SYMBOL_THREAD_LAST,
        Value::Macro(Rc::new(RefCell::new(ThreadLastMacro))),
    );
    let _ = env.insert(&SYMBOL_COND, Value::Macro(Rc::new(RefCell::new(CondMacro))));
    let _ = env.insert(&SYMBOL_AND, Value::Macro(Rc::new(RefCell::new(AndMacro))));
    let _ = env.insert(&SYMBOL_OR, Value::Macro(Rc::new(RefCell::new(OrMacro))));
    let _ = env.insert(&SYMBOL_FOR, Value::Macro(Rc::new(RefCell::new(ForMacro))));
    let _ = env.insert(
        &SYMBOL_MACRO,
        Value::Macro(Rc::new(RefCell::new(MacroMacro))),
    );
}
