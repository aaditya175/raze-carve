use crate::error::CoreError;
use mlua::{prelude::*, LuaOptions, StdLib};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct PluginSignature {
    pub name: String,
    pub category: String,
    pub header: Vec<u8>,
    pub footer: Option<Vec<u8>>,
    pub max_size: u64,
    pub has_validator: bool,
}

pub struct LuaPlugin {
    pub signature: PluginSignature,
    script_source: String,
}

pub struct LuaPluginHost {
    plugins: Vec<LuaPlugin>,
}

impl Default for LuaPluginHost {
    fn default() -> Self {
        Self::new()
    }
}

impl LuaPluginHost {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    pub fn load_script(&mut self, script_path: &Path) -> Result<(), CoreError> {
        let script_source = fs::read_to_string(script_path).map_err(CoreError::Io)?;
        self.load_script_content(&script_source)
    }

    /// Load a sandboxed Lua plugin from raw script source text.
    /// If a plugin with the same signature name already exists, it is replaced.
    pub fn load_script_content(&mut self, script_source: &str) -> Result<(), CoreError> {
        // SAFETY-CRITICAL: plugins are untrusted third-party scripts.
        // Only load string/table/math — NOT io, os, or package (which allow
        // filesystem access, process execution, and loading native libs).
        let safe_libs = StdLib::STRING | StdLib::TABLE | StdLib::MATH;
        let lua = Lua::new_with(safe_libs, LuaOptions::default())
            .map_err(|e| CoreError::Parse(format!("Failed to init sandboxed Lua: {e}")))?;

        // DoS guard: limit execution instructions during script evaluation
        lua.set_hook(
            mlua::HookTriggers::default().every_nth_instruction(100_000),
            |_lua, _debug| {
                Err(mlua::Error::RuntimeError(
                    "Lua execution instruction limit exceeded (100,000 instructions) during load — aborted".to_string(),
                ))
            },
        );

        let globals = lua.globals();

        // Define signature function to capture data
        let _sig_data = lua
            .create_table()
            .map_err(|e| CoreError::Parse(e.to_string()))?;

        let signature_func = lua
            .create_function(move |lua, table: mlua::Table| {
                let name: String = table.get("name")?;
                let category: String = table.get("category")?;
                let header_val: mlua::Value = table.get("header")?;
                let footer_val: Option<mlua::Value> = table.get("footer").ok();

                let max_size: u64 = match table.get::<_, u64>("max_size") {
                    Ok(v) => v,
                    Err(_) => match table.get::<_, String>("max_size") {
                        Ok(s) => {
                            crate::plugins::toml_loader::parse_size_string(&s).map_err(|e| {
                                mlua::Error::RuntimeError(format!("Invalid max_size: {}", e))
                            })?
                        }
                        Err(_) => 100 * 1024 * 1024,
                    },
                };
                let has_validate: bool = table.contains_key("validate")?;

                let parse_bytes_value = |val: mlua::Value| -> Result<Vec<u8>, mlua::Error> {
                    match val {
                        mlua::Value::String(s) => {
                            let bytes = s.as_bytes();
                            if let Ok(text) = std::str::from_utf8(bytes) {
                                let clean = text.replace(' ', "");
                                if !clean.is_empty()
                                    && clean.len().is_multiple_of(2)
                                    && clean.chars().all(|c| c.is_ascii_hexdigit())
                                {
                                    return (0..clean.len())
                                        .step_by(2)
                                        .map(|i| u8::from_str_radix(&clean[i..i + 2], 16))
                                        .collect::<Result<Vec<u8>, _>>()
                                        .map_err(|e| {
                                            mlua::Error::RuntimeError(format!("Invalid hex: {}", e))
                                        });
                                }
                            }
                            Ok(bytes.to_vec())
                        }
                        mlua::Value::Table(t) => {
                            let mut vec = Vec::new();
                            for i in 1..=t.raw_len() {
                                let b: u8 = t.raw_get(i)?;
                                vec.push(b);
                            }
                            Ok(vec)
                        }
                        _ => Err(mlua::Error::RuntimeError(
                            "header/footer must be a hex string, binary string, or byte table"
                                .to_string(),
                        )),
                    }
                };

                let header = parse_bytes_value(header_val)?;
                let footer = match footer_val {
                    Some(mlua::Value::Nil) | None => None,
                    Some(v) => Some(parse_bytes_value(v)?),
                };

                // Store bytes as raw Lua strings
                let res = lua.create_table()?;
                res.set("name", name)?;
                res.set("category", category)?;
                let lua_header = lua.create_string(&header)?;
                res.set("header", lua_header)?;
                if let Some(ref f) = footer {
                    let lua_footer = lua.create_string(f)?;
                    res.set("footer", lua_footer)?;
                }
                res.set("max_size", max_size)?;
                res.set("has_validator", has_validate)?;
                Ok(res)
            })
            .map_err(|e| CoreError::Parse(e.to_string()))?;

        globals
            .set("signature", signature_func)
            .map_err(|e| CoreError::Parse(e.to_string()))?;

        // Evaluate the script
        let chunk = lua.load(script_source);
        let result: mlua::Table = chunk.eval().map_err(|e| CoreError::Parse(e.to_string()))?;

        let name: String = result.get("name").unwrap_or_default();
        let category: String = result.get("category").unwrap_or_default();
        let header: Vec<u8> = result
            .get::<_, mlua::String>("header")
            .map(|s| s.as_bytes().to_vec())
            .unwrap_or_default();
        let footer: Option<Vec<u8>> = result
            .get::<_, Option<mlua::String>>("footer")
            .ok()
            .flatten()
            .map(|s| s.as_bytes().to_vec());
        let max_size: u64 = result.get("max_size").unwrap_or(0);
        let has_validator: bool = result.get("has_validator").unwrap_or(false);

        let plugin_sig = PluginSignature {
            name: name.clone(),
            category,
            header,
            footer,
            max_size,
            has_validator,
        };

        if let Some(idx) = self.plugins.iter().position(|p| p.signature.name == name) {
            self.plugins[idx] = LuaPlugin {
                signature: plugin_sig,
                script_source: script_source.to_string(),
            };
        } else {
            self.plugins.push(LuaPlugin {
                signature: plugin_sig,
                script_source: script_source.to_string(),
            });
        }

        Ok(())
    }

    pub fn loaded_signatures(&self) -> Vec<&PluginSignature> {
        self.plugins.iter().map(|p| &p.signature).collect()
    }

    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }

    pub fn has_plugin(&self, name: &str) -> bool {
        self.plugins.iter().any(|p| p.signature.name == name)
    }

    pub fn validate(&self, plugin_name: &str, data: &[u8]) -> Result<bool, CoreError> {
        let plugin = match self
            .plugins
            .iter()
            .find(|p| p.signature.name == plugin_name)
        {
            Some(p) => p,
            None => return Ok(true),
        };

        if !plugin.signature.has_validator {
            return Ok(true);
        }

        let safe_libs = StdLib::STRING | StdLib::TABLE | StdLib::MATH;
        let lua = Lua::new_with(safe_libs, LuaOptions::default())
            .map_err(|e| CoreError::Parse(format!("Failed to init sandboxed Lua: {e}")))?;

        // Set instruction hook limit (100k instructions) to prevent infinite loop DoS
        lua.set_hook(
            mlua::HookTriggers::default().every_nth_instruction(100_000),
            |_lua, _debug| {
                Err(mlua::Error::RuntimeError(
                    "Lua execution instruction limit exceeded (100,000 instructions) — execution terminated to prevent DoS".to_string()
                ))
            },
        );

        let globals = lua.globals();

        let signature_func = lua
            .create_function(move |_lua, table: mlua::Table| {
                Ok(table) // just return the table during validation
            })
            .map_err(|e| CoreError::Parse(e.to_string()))?;

        globals
            .set("signature", signature_func)
            .map_err(|e| CoreError::Parse(e.to_string()))?;

        let chunk = lua.load(&plugin.script_source);
        let result: mlua::Table = chunk.eval().map_err(|e| CoreError::Parse(e.to_string()))?;

        let validate_func: mlua::Function = result
            .get("validate")
            .map_err(|_| CoreError::Parse("validate function not found".to_string()))?;

        // Pass raw bytes as a Lua string (Lua strings are byte arrays, not
        // UTF-8 text) — from_utf8_lossy previously corrupted binary magic
        // bytes (JPEG/PNG/ZIP headers) before the validator ever saw them.
        let lua_bytes = lua
            .create_string(data)
            .map_err(|e| CoreError::Parse(e.to_string()))?;
        let val: mlua::Value = validate_func
            .call(lua_bytes)
            .map_err(|e| CoreError::Parse(e.to_string()))?;

        let is_valid = match val {
            mlua::Value::Boolean(b) => b,
            mlua::Value::Nil => false,
            _ => true,
        };

        Ok(is_valid)
    }

    pub fn load_directory(&mut self, dir: &Path) -> Result<usize, CoreError> {
        let mut count = 0;
        if dir.is_dir() {
            for entry in fs::read_dir(dir).map_err(CoreError::Io)? {
                let entry = entry.map_err(CoreError::Io)?;
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("lua") {
                    match self.load_script(&path) {
                        Ok(()) => count += 1,
                        Err(e) => {
                            tracing::warn!(
                                "Failed to load Lua plugin from {}: {}",
                                path.display(),
                                e
                            );
                        }
                    }
                }
            }
        }
        Ok(count)
    }
}
