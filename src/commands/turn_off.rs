use crate::config::{load_local_config, save_local_config};
use anyhow::Result;

pub fn cmd_turn_off() -> Result<()> {
    let Some(mut cfg) = load_local_config()? else {
        println!("Settings Sync is already off.");
        return Ok(());
    };
    cfg.enabled = false;
    save_local_config(&cfg)?;
    println!("Settings Sync is off on this machine.");
    println!("Cloud data is kept. Run `zync on` to turn it back on.");
    Ok(())
}
