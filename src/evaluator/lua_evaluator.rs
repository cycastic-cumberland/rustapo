use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use log::{Level, log};
use rlua::{Function, Lua, Nil};
use rlua::prelude::LuaError;
use crate::evaluator::evaluator::{BooleanEvaluationResult, IRustapoEvaluator, IRustapoWrappedInstance};
use crate::evaluator::typed_variadic_arguments::TypedLuaVariadicArguments;

pub struct LuaEvaluator {
    runtime: Rc<Lua>
}

pub struct LuaEvaluatorInstance {
    runtime: Rc<Lua>,
    declared: RefCell<HashSet<String>>
}

fn lua_log(mut log_level: Level, messages: &Vec<String>) {
    let mut builder = string_builder::Builder::default();
    for msg in messages {
        builder.append(msg.clone());
        builder.append(" ");
    }
    builder.append("\n");
    let message = match builder.string() {
        Ok(v) => { v }
        Err(e) => { log_level = Level::Error; format!("UTF-8 error encountered while logging: {}", e) }
    };
    log!(log_level, "{}", message);
}

impl LuaEvaluator {
    pub fn new() -> Self {
        let rt = LuaEvaluator{
            runtime: Rc::new(Lua::new())
        };
        match rt.runtime.context(|lua_ctx| -> Result<(), LuaError> {
            let info_closure: Function = lua_ctx.create_function(|_, vec: TypedLuaVariadicArguments<String>| {
                lua_log(Level::Info, vec.get_va_args());
                Ok(())
            })?;
            let debug_closure: Function = lua_ctx.create_function(|_, vec: TypedLuaVariadicArguments<String>| {
                lua_log(Level::Debug, vec.get_va_args());
                Ok(())
            })?;
            let warn_closure: Function = lua_ctx.create_function(|_, vec: TypedLuaVariadicArguments<String>| {
                lua_log(Level::Warn, vec.get_va_args());
                Ok(())
            })?;
            let min_closure: Function = lua_ctx.create_function(|_, vec: TypedLuaVariadicArguments<f64>| {
                let mut curr_min = f64::MAX;
                for x in vec.get_va_args() {
                    curr_min = if *x < curr_min { *x } else { curr_min }
                }
                Ok(curr_min)
            })?;
            let max_closure: Function = lua_ctx.create_function(|_, vec: TypedLuaVariadicArguments<f64>| {
                let mut curr_max = f64::MAX;
                for x in vec.get_va_args() {
                    curr_max = if *x < curr_max { curr_max } else { *x }
                }
                Ok(curr_max)
            })?;
            let avg_closure: Function = lua_ctx.create_function(|_, vec: TypedLuaVariadicArguments<f64>| {
                let va_args = vec.get_va_args();
                let mut ret = 0f64;
                if va_args.len() == 0 {
                    return Ok(ret);
                }
                let mut sum = 0f64;
                for x in va_args {
                    sum += *x;
                }
                ret = sum / va_args.len() as f64;
                return Ok(ret);
            })?;
            let sum_closure: Function = lua_ctx.create_function(|_, vec: TypedLuaVariadicArguments<f64>| {
                let va_args = vec.get_va_args();
                let mut sum = 0f64;
                for x in va_args {
                    sum += *x;
                }
                Ok(sum)
            })?;
            lua_ctx.globals().set("info", info_closure)?;
            lua_ctx.globals().set("debug", debug_closure)?;
            lua_ctx.globals().set("warn", warn_closure)?;
            lua_ctx.globals().set("min", min_closure)?;
            lua_ctx.globals().set("max", max_closure)?;
            lua_ctx.globals().set("avg", avg_closure)?;
            lua_ctx.globals().set("sum", sum_closure)?;
            Ok(())
        }){
            Ok(_) => { () },
            Err(e) => { panic!("Failed to setup LuaEvaluator: {}", e.to_string()) }
        };
        rt
    }
}

impl LuaEvaluatorInstance {
    pub(crate) fn new(lua: Rc<Lua>) -> Self {
        Self {
            runtime: lua,
            declared: RefCell::new(HashSet::new())
        }
    }
}

impl IRustapoWrappedInstance for LuaEvaluatorInstance {
    fn evaluate_boolean(&self, expr: &String) -> BooleanEvaluationResult {
        match self.runtime.context(move |lua_ctx| -> Result<bool, LuaError> {
            let e = &**expr;
            let result = lua_ctx.load(e).eval::<bool>()?;
            Ok(result)
        }){
            Ok(v) => {
                BooleanEvaluationResult::create_completed(v)
            }
            Err(e) => {
                BooleanEvaluationResult::create_faulty(Box::new(e))
            }
        }
    }

    fn declare_f64(&self, name: &String, value: f64) -> Result<(), crate::evaluator::evaluator::Error> {
        {
            let mut set = self.declared.borrow_mut();
            set.insert(name.clone());
        }
        self.runtime.context(move |lua_ctx| {
            let e = &**name;
            lua_ctx.globals().set(e, value)?;
            Ok(())
        })
    }

    fn declare_string(&self, name: &String, value: String) -> Result<(), crate::evaluator::evaluator::Error> {
        {
            let mut set = self.declared.borrow_mut();
            set.insert(name.clone());
        }
        self.runtime.context(move |lua_ctx| {
            let e = &**name;
            lua_ctx.globals().set(e, value)?;
            Ok(())
        })
    }
}

impl Drop for LuaEvaluatorInstance {
    fn drop(&mut self) {
        let set = self.declared.borrow();
        let _ = self.runtime.context(move |lua_ctx| -> Result<(), rlua::Error> {
            let globals = lua_ctx.globals();
            for x in set.iter(){
                globals.set(&**x, Nil)?;
            }
            Ok(())
        });
    }
}

impl IRustapoEvaluator<LuaEvaluatorInstance> for LuaEvaluator {
    fn capture(&self) -> LuaEvaluatorInstance {
        LuaEvaluatorInstance::new(self.runtime.clone())
    }
}

impl Default for LuaEvaluator {
    fn default() -> Self {
        Self::new()
    }
}