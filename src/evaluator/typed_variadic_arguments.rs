use rlua::{Result, FromLua, FromLuaMulti, MultiValue, Context};

pub struct TypedLuaVariadicArguments<T>
    where T: for<'a> FromLua<'a>
{
    args: Vec<T>
}

impl<T> FromLuaMulti<'_> for TypedLuaVariadicArguments<T>
    where T: for<'a> FromLua<'a> + Clone
{
    fn from_lua_multi<'lua>(values: MultiValue<'lua>, lua: Context<'lua>) -> Result<Self> {
        let mut ret: Vec<T> = Vec::with_capacity(values.len());
        for x in values {
            let casted: T = T::from_lua(x, lua)?;
            ret.push(casted);
        }
        Ok(TypedLuaVariadicArguments {
            args: ret
        })
    }
}

impl<T> TypedLuaVariadicArguments<T>
    where T: for<'a> FromLua<'a> + Clone
{
    // pub fn get_mut_va_args(&self) -> Vec<T> {
    //     self.args.clone()
    // }
    pub fn get_va_args(&self) -> &Vec<T> {
        &self.args
    }
}
