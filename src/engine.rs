use mlua::{IntoLua, Lua};

const INIT_FUNCTION_NAME: &str = "init";
const UPDATE_FUNCTION_NAME: &str = "update";
const DRAW_FUNCTION_NAME: &str = "draw";

pub struct LuaEngine {
    lua: Lua,
}

impl LuaEngine {
    /// Creates a new instance of the engine with the provided main code.
    pub fn new(code: String) -> mlua::Result<Self> {
        let lua = Lua::new();

        lua.load(code).exec()?;

        Ok(Self { lua })
    }

    pub fn init(&mut self) -> mlua::Result<()> {
        if let Ok(f) = &self
            .lua
            .globals()
            .get::<&str, mlua::Function>(INIT_FUNCTION_NAME)
        {
            f.call::<_, ()>(())?
        }

        Ok(())
    }

    pub fn update(&mut self, delta_time: f64) -> mlua::Result<()> {
        if let Ok(f) = &mut self
            .lua
            .globals()
            .get::<&str, mlua::Function>(UPDATE_FUNCTION_NAME)
        {
            f.call::<_, ()>(delta_time)?
        }

        Ok(())
    }

    pub fn draw(&mut self) -> mlua::Result<()> {
        if let Ok(f) = &self
            .lua
            .globals()
            .get::<&str, mlua::Function>(DRAW_FUNCTION_NAME)
        {
            f.call::<_, ()>(())?
        }

        Ok(())
    }

    pub fn set_global<'lua, V>(&'lua mut self, name: &'static str, value: V) -> mlua::Result<()>
    where
        V: mlua::IntoLua<'lua>,
    {
        self.lua.globals().set(name, value)
    }

    pub fn provide_function<'lua, F>(
        &'lua mut self,
        name: &'static str,
        function: F,
    ) -> mlua::Result<()>
    where
        F: IntoLua<'lua>,
    {
        self.lua.globals().set(name, function)
    }
}
