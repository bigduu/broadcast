use auto_launch::AutoLaunchBuilder;
use tracing::{error, info};

pub fn auto_launch_self() {
    // Get the program location
    if let Ok(path) = std::env::current_exe() {
        info!("Program path: {}", path.to_str().unwrap());
        let lnk = create_lnk(path.to_str().unwrap().to_string());
        match AutoLaunchBuilder::new()
            .set_app_name("broadcast_start_up")
            .set_app_path(&lnk)
            .set_use_launch_agent(true)
            .build()
        {
            Ok(auto) => {
                if auto.is_enabled().unwrap() {
                    info!("Auto launch is enabled");
                    let _ = auto.disable();
                    let _ = auto.enable();
                } else {
                    info!("Auto launch is disabled");
                    let _ = auto.enable();
                }
            }
            Err(e) => {
                error!("Error: {}", e);
            }
        }
    }
}

fn create_lnk(exe_path: String) -> String {
    format!("{}{}", exe_path, ".lnk")
}
