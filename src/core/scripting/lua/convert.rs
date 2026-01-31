use mlua::prelude::{Lua, LuaTable, LuaValue, LuaError, LuaResult};
use crate::core::messaging::{self, JSONObject};

pub fn convert_to_json(lua: &Lua, table: &LuaTable) -> LuaResult<JSONObject> {
    let object_value = lua_table_to_json(lua, table)?;
    if let serde_json::Value::Object(map) = object_value {
        Ok(map)
    } else {
        Err(LuaError::FromLuaConversionError {
            from: "LuaTable",
            to: "messaging::JSONObject".to_string(),
            message: Some("Expected JSON Object".to_string()),
        })
    }
}

fn lua_table_to_json(lua: &Lua, table: &LuaTable) -> LuaResult<serde_json::Value> {
    let mut map = serde_json::Map::new();

    // Is lua table an array?
    if is_table_array(table)? {
        let mut arr = Vec::new();
        for pair in table.pairs::<LuaValue, LuaValue>() {
            let (_, value) = pair?;
            let json_value = lua_to_json_value(lua, &value)?;
            arr.push(json_value);
        }
        return Ok(serde_json::Value::Array(arr));
    }


    for pair in table.pairs::<LuaValue, LuaValue>() {
        let (key, value) = pair?;

        let key_str = match key {
            LuaValue::String(s) => s.to_str()?.to_string(),
            LuaValue::Integer(i) => i.to_string(),
            _ => continue,
        };

        let json_value = lua_to_json_value(lua, &value)?;
        map.insert(key_str, json_value);
    }

    Ok(serde_json::Value::Object(map))
}


fn lua_to_json_value(lua: &Lua, value: &LuaValue) -> LuaResult<serde_json::Value> {
    let json_value = match value {
        LuaValue::String(s) => serde_json::Value::String(s.to_str()?.to_string()),
        LuaValue::Integer(i) => serde_json::Value::Number((*i).into()),
        LuaValue::Number(n) => serde_json::Number::from_f64(*n)
            .map_or(serde_json::Value::Null, serde_json::Value::Number),
        LuaValue::Boolean(b) => serde_json::Value::Bool(*b),
        LuaValue::Table(t) => lua_table_to_json(lua, t)?,
        _ => serde_json::Value::Null,
    };

    Ok(json_value)
}

fn is_table_array(table: &LuaTable) -> LuaResult<bool> {
    let mut index = 1;
    if table.len()? == 0 {
        return Ok(false);
    }

    for pair in table.pairs::<LuaValue, LuaValue>() {
        let (key, _) = pair?;
        match key {
            LuaValue::Integer(i) if i == index => index += 1,
            _ => return Ok(false),
        }
    }

    Ok(true)
}

pub fn convert_to_lua_table(lua: &Lua, object: &messaging::JSONObject) -> LuaResult<LuaValue> {
    let table = lua.create_table()?;

    for (key, value) in object {
        let lua_value = convert_json_to_lua_value(lua, value)?;
        table.set(key.as_str(), lua_value)?;
    }

    Ok(LuaValue::Table(table))
}

fn convert_json_to_lua_value(lua: &Lua, value: &serde_json::Value) -> LuaResult<LuaValue> {
    let lua_value = match value {
        serde_json::Value::String(s) => LuaValue::String(lua.create_string(s)?),
        serde_json::Value::Bool(b) => LuaValue::Boolean(*b),
        serde_json::Value::Object(map) => convert_to_lua_table(lua, map)?,
        serde_json::Value::Array(arr) => {
            let table = lua.create_table()?;

            for (index, val) in arr.iter().enumerate() {
                table.set(index + 1, convert_json_to_lua_value(lua, val)?)?; // Lua arrays are 1-indexed
            }
            LuaValue::Table(table)
        }

        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                LuaValue::Integer(i)
            } else if let Some(f) = n.as_f64() {
                LuaValue::Number(f)
            } else {
                LuaValue::Nil
            }
        }
        _ => {
            return Err(LuaError::FromLuaConversionError {
                from: "serde_json::Value",
                to: "LuaTable".to_string(),
                message: Some("Expected JSON Object or Array".to_string()),
            });
        }
    };

    Ok(lua_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversion_to_lua() {
        let lua = Lua::new();

        let json_obj: messaging::JSONObject = serde_json::json!({
            "name": "entity_1",
            "age": 5,
            "is_active": true,
            "attributes": {
                "strength": 10,
                "agility": 8
            },
            "tags": ["npc", "friendly"]
        })
        .as_object()
        .unwrap()
        .clone();

        let lua_table = convert_to_lua_table(&lua, &json_obj).unwrap();
        if let LuaValue::Table(table) = lua_table {
            let converted_json = convert_to_json(&lua, &table).unwrap();
            assert_eq!(json_obj, converted_json);
        } else {
            panic!("Expected LuaValue::Table");
        }
    }

    #[test]
    fn test_nested_arrays_conversion() {
        let lua = Lua::new();

        let json_obj: messaging::JSONObject = serde_json::json!({
            "matrix": [
                [1, 2, 3],
                [4, 5, 6],
                [7, 8, 9]
            ]
        })
        .as_object()
        .unwrap()
        .clone();

        let lua_table = convert_to_lua_table(&lua, &json_obj).unwrap();
        if let LuaValue::Table(table) = lua_table {
            let converted_json = convert_to_json(&lua, &table).unwrap();
            assert_eq!(json_obj, converted_json);
        } else {
            panic!("Expected LuaValue::Table");
        }
    }
}