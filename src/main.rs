mod ecs;
mod lua;
mod grid;

use mlua::prelude::*;

fn main() -> LuaResult<()>  {
    let lua = Lua::new();

    let map_table = lua.create_table()?;
    map_table.set(1, "one")?;
    map_table.set("two", 2)?;

    let mut world = ecs::World::new();

    let mut luaSystem = Box::new(lua::LuaScriptSystem::new());
    let mut gridSystem = Box::new(grid::GridSystem::new());



  

    world.add_system(luaSystem);
    world.add_system(gridSystem);

    world.add_component(0, Box::new(grid::GridComponent::new(100, 200)));
    world.add_component(1, Box::new(lua::LuaScriptComponent::new(1, String::from("print('Hello from Lua script!')"))));


    world.update(0);


    lua.globals().set("map_table", map_table)?;
    lua.load("for k,v in pairs(map_table) do print(k,v) end").exec()?;

    println!("Hello, world!");

    Ok(())
}
