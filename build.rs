use std::env;
use winresource::WindowsResource;
fn main() {
    slint_build::compile("ui/app-window.slint").unwrap();
    if env::var_os("CARGO_CFG_WINDOWS").is_some() {

            let result = WindowsResource::new().set_icon("cube-pro.ico").compile();
            if result.is_err() {
                println!("{}", result.unwrap_err());
            }
    }
}
