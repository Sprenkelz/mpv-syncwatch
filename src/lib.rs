mod plugin;
mod config;

use config::Config;
use plugin::Plugin;
use mpv_client_dyn::mpv_handle;

#[unsafe(no_mangle)]
extern "C" fn mpv_open_cplugin(handle: *mut mpv_handle) -> std::os::raw::c_int {
    if let Err(e) = env_logger::try_init() {
        eprintln!("Failed to initialize logger: {}", e);
    }

    let config = match Config::get() {
        Some(cfg) => cfg,
        None => return -1,
    };

    let mut client = Plugin::new(handle, config);

    log::trace!("Starting syncwatch [{}]", client.name());

    match client.start() {
        Ok(_) => {
            log::trace!("Closing syncwatch [{}]", client.name());
            0
        }
        Err(e) => {
            log::error!("Unrecoverable error on plugin syncwatch [{}]: {}", client.name(), e);
            -1
        }
    }
}