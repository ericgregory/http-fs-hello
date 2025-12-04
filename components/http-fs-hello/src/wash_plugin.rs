use anyhow::Result;

mod bindings {
    use super::Component;
    wit_bindgen::generate!({
        world: "wash-plugin",
        generate_all,
    });
    export!(Component);
}
use bindings::exports::wasmcloud::wash::plugin;
use bindings::wasmcloud::wash::types::{Command, HookType, Metadata, Runner};

struct Component;

// We implement plugin for this component (along with HTTP via wstd)
// so that when loaded by wasmCloud, it will be able to use attached volumes. 
impl plugin::Guest for Component {
    /// Called by wash to retrieve the plugin metadata
    fn info() -> Metadata {
        Metadata {
            id: "examples.wasmcloud.http-fs-hello".to_string(),
            name: "http-fs-hello-plugin".to_string(),
            description: "Example HTTPS service that uses the wasi:filesystem API".to_string(),
            contact: "wasmCloud Team".to_string(),
            url: "https://github.com/ericgregory/http-fs-hello/tree/main".to_string(),
            license: "Apache-2.0".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            command: None,
            sub_commands: vec![],
            hooks: vec![HookType::DevRegister],
        }
    }

    fn initialize(_: Runner) -> Result<String, String> {
        Ok(String::with_capacity(0))
    }

    fn run(_: Runner, _: Command) -> Result<String, String> {
        Err("no command registered".to_string())
    }

    fn hook(_: Runner, _: HookType) -> Result<String, String> {
        Err("invalid hook usage".to_string())
    }
}
