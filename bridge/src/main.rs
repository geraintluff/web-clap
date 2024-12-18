use std::path::{Path, PathBuf};

use anyhow::bail;
use clap::{Arg, Parser};
use log::info;
use wasmer::{imports, Instance, Module, Store, Value};
use wasmer_wasix::{Pipe, WasiEnv};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct CommandLineArguments {
    /// Name of the person to greet
    #[arg(short, long, required(true))]
    plugin: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cmd_args = CommandLineArguments::parse();

    let mut store = Store::default();
    let module = Module::from_file(&store, PathBuf::from(&cmd_args.plugin))?;

    let (instance, env) = WasiEnv::builder("test clap host").instantiate(module, &mut store)?;

    for (symbol, ext) in &instance.exports {
        println!("Global Symbol: {}, {:?}", symbol, ext);
    }

    let entry = instance.exports.get_global("clap_entry")?;
    let clap_entry_value = entry.get(&mut store);
    println!("Found value of type: {:?}", clap_entry_value);

    let Some(v1) = clap_entry_value.externref() else {
        bail!("couldn't get clap_entry value 1");
    };

    let Some(entry_value) = v1 else {
        bail!("couldn't get clap_entry value 2");
    };

    let Some(entry) = entry_value.downcast::<clap_sys::entry::clap_plugin_entry>(&store) else {
        bail!("couldn't get clap_entry value 3");
    };

    unsafe {
        if let Some(init) = entry.init {
            if !init(c"test-wasm-host".as_ptr()) {
                bail!("couldn't initialize clap plugin");
            }
        }

        info!(
            "clap plugin version: {}.{}.{}",
            entry.clap_version.major, entry.clap_version.minor, entry.clap_version.revision
        );

        entry.deinit.map(|fct| fct());
    }

    // let add_one = instance.exports.get_function("add_one")?;
    // let result = add_one.call(&mut store, &[Value::I32(42)])?;
    // assert_eq!(result[0], Value::I32(43));

    Ok(())
}
