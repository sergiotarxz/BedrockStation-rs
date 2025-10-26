#![windows_subsystem = "windows"]

use std::error::Error;
use std::env::set_var;
use std::fs::File;
use std::io::Write;
use std::io::BufReader;
use std::io::BufRead;
use std::env;
use regex::Regex;
use interprocess::local_socket::traits::Stream;
use slint::SharedString;
use std::net::UdpSocket;
use std::collections::HashMap;
use slint::CloseRequestResponse;
use slint::run_event_loop;
use slint::set_xdg_app_id;
use process_alive::Pid;
use process_alive::State;
use std::io::Read;
use slint::TimerMode;
use slint::Timer;
use interprocess::local_socket::ListenerOptions;
use interprocess::local_socket::ListenerNonblockingMode;
use interprocess::local_socket::ToNsName;
use interprocess::local_socket::GenericNamespaced;
use interprocess::local_socket::Name;
use interprocess::local_socket;
use interprocess::local_socket::traits::Listener;
use std::thread;
use std::sync::Arc;
use std::option::Option;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::mpsc::channel;
use std::sync::mpsc::TryRecvError;
use std::sync::Mutex;
use std::thread::JoinHandle;
use std::net::SocketAddr::V4;
use std::net::SocketAddrV4;

slint::include_modules!();

static mut SHOW_NOW: bool = false;


fn main() -> Result<(), Box<dyn Error>> {
    start_or_show();
    Ok(())
}

fn start_or_show() {
    if is_started() {
        println!("Using the old process to resume…");
        show();
        return;
    }
    start();
}

fn get_unix_name() -> Name<'static> {
    let name = ".bedrockstation.sock".to_ns_name::<GenericNamespaced>();
    if let Err(error) = name {
        panic!("Could not convert to NS Name {error}");
    }
    return name.unwrap().into_owned();
}

fn unix_start_server() -> local_socket::Listener {
    let name = get_unix_name();
    let opts = ListenerOptions::new().name(name).nonblocking(ListenerNonblockingMode::Both);
    let listener = opts.create_sync();
    if let Err(error) = listener {
        panic!("Couldn't listen for requests: {error}");
    }
    return listener.unwrap();
}

fn pid_file() -> String {
    return home() + "/bedrockstation.pid";
}

fn home() -> String {
    let mut home = env::var("HOME");
    if home.is_err() {
        home = env::var("USERPROFILE");
    }
    if home.is_err() {
        panic!("No home found!!!");
    }
    return home.unwrap();
}

fn is_started() -> bool {
    let file = File::open(pid_file());
    if let Err(error) = file {
        println!("{error}");
        return false;
    }
    let mut file = file.unwrap();
    let mut contents = String::new();
    if let Err(error) = file.read_to_string(&mut contents) {
        println!("{error}");
        return false;
    }
    let stored_pid = contents.parse::<u32>();
    if let Err(error) = stored_pid {
        println!("{error}");
        return false;
    }
    let stored_pid = stored_pid.unwrap();
    let stored_pid = Pid::from(stored_pid);
    let state = process_alive::state(stored_pid);
    if state != State::Alive {
        println!("Defuct old process");
        return false;
    }
    return true;
}

fn show() {
    let name = get_unix_name();
    let conn = local_socket::Stream::connect(name);
    if let Err(error) = conn {
        panic!("Could not connect to app {error}");
    }
    let mut conn = BufReader::new(conn.unwrap());
    let _ = conn.get_mut().write_all(b"show\n");
}

fn start() {
    if let Ok(mut pid_file) = File::create(pid_file()) {
        if let Err(error) = pid_file.write_all(std::process::id().to_string().as_bytes()) {
            panic!("{error}");
        }
    }
    unsafe {
        set_var("SLINT_BACKEND", "winit-software");
    };
    let ui_boxed = Box::new(AppWindow::new().unwrap());
    let ui = ui_boxed.as_ref();
    let _ = set_xdg_app_id("me.sergiotarxz.bedrockstation.rust");
    // Has to be on scope to avoid timer to dissapear
    let _timer = set_callbacks(ui);
    set_button_stopped(ui);
    let result = ui.show();
    if let Err(error) = result {
        println!("{}", error);
    }
    loop {
        let result = run_event_loop();
        if let Err(error) = result {
            println!("{}", error);
        }
    }
}

fn start_server(ui: &AppWindow, rx: Arc<Mutex<Receiver<()>>>, join: &Arc<Mutex<Option<JoinHandle<()>>>>) {
    if is_server_started(&join) {
        return;
    }
    let server_address = ui.get_server_address();
    let server_port = ui.get_port();
    // Do not use in the thread the JOIN variable.
    let rx_closure  = Arc::clone(&rx);
    let mut join_lock = join.lock().unwrap();
    *join_lock = Some(thread::spawn(move || {
        let rx = rx_closure.lock().unwrap();
        let socket = UdpSocket::bind("0.0.0.0:".to_owned()+server_port.as_str());
        let socket = socket.unwrap();
        let _ = socket.set_nonblocking(true);
        let mut client_to_server = HashMap::<SocketAddrV4, UdpSocket>::new();
        loop {
            let recv = (*rx).try_recv();
            if let Ok(_) = recv {
                break;
            }
            if let Err(TryRecvError::Disconnected) = recv {
                break;
            }
            for _i in 0..5 {
                let mut buffer = [0; 1024];
                if let Ok((amt, src)) = socket.recv_from(&mut buffer) {
                    let buffer = &mut buffer[..amt];
                    if let V4(src) = src {
                        if let None = client_to_server.get(&src) {
                            let server_socket = UdpSocket::bind("0.0.0.0:".to_owned()+ &src.port().to_string());
                            let server_socket = server_socket.unwrap();
                            let _ = server_socket.set_nonblocking(true);
                            let _ = server_socket.connect(server_address.as_str().to_owned()+":"+server_port.as_str()); 
                            client_to_server.insert(src, server_socket); 
                        }
                        if let Some(server_socket) = client_to_server.get(&src) {
                            let _ = server_socket.send_to(buffer, server_address.as_str().to_owned()+":"+server_port.as_str());
                        }
                    }
                }
            }
            for (src, server_socket) in client_to_server.iter() {
                for _i in 0..5 {
                    let mut buffer = [0; 1024]; 
                    if let Ok((amt, _)) = server_socket.recv_from(&mut buffer) {
                        let buffer = &mut buffer[..amt];
                        let _ = socket.send_to(buffer, src);
                    }
                }
            }
        }
        println!("Finishing proxy");
    }));
}

fn set_button_stopped(ui: &AppWindow) {
    ui.set_start_stop_button_string(SharedString::new() + "Iniciar el proxy");
}

fn set_button_started(ui: &AppWindow) {
    ui.set_start_stop_button_string(SharedString::new() + "Terminar el proxy");
}

fn set_timer(ui: &AppWindow, join: &Arc<Mutex<Option<JoinHandle<()>>>>) -> Timer {
    let timer = Timer::default();
    let weak = ui.as_weak();
    let listener = unix_start_server();
    let join = Arc::clone(&join);
    timer.start(TimerMode::Repeated, std::time::Duration::from_millis(100), move || {
        let ui = weak.unwrap();
        unsafe {
            if SHOW_NOW {
                let _ = ui.show();
                SHOW_NOW = false;
            }
        }
        change_server_started_status(&ui, &join);
        let mut buffer = String::with_capacity(128);
        let mut i = 5;
        loop {
            let conn = listener.accept();
            if let Err(_error) = conn {
                // TODO: Warning of errors here would pollute too much the console log
                // Find a good way to do so.
                break;
            }
            let conn = conn.unwrap();
            let mut conn = BufReader::new(conn);
            while let Ok(_ok) = conn.read_line(&mut buffer) {
                buffer = buffer.trim_end_matches(|c| c == '\n' || c == '\r').to_string();
                if buffer == "show" {
                    println!("Received instructions to resume the window");
                    let _ = ui.show();
                    buffer = "".to_string();
                    break;
                }
                buffer = "".to_string();
            }
            i -= 1;
            if i == 0 {
                break;
            }
        }
    });
    timer
}

static mut IS_STARTED_LAST: bool = false;

fn change_server_started_status(ui: &AppWindow, join: &Arc<Mutex<Option<std::thread::JoinHandle<()>>>>) {
    unsafe {
        if is_server_started(join) {
            if IS_STARTED_LAST {
                return;
            }
            IS_STARTED_LAST = true;
            set_button_started(&ui);
            ui.set_server_started(false);
        } else {
            if !IS_STARTED_LAST {
                return;
            }
            IS_STARTED_LAST = false;
            set_button_stopped(&ui);
            ui.set_server_started(true);
        }
        let _ = ui.hide();
        SHOW_NOW = true;
    }
}

fn is_server_started(join: &Arc<Mutex<Option<std::thread::JoinHandle<()>>>>) -> bool {
    let join_lock = join.lock().unwrap();
    if let None = &*join_lock {
        return false;
    }
    if let Some(join) = &*join_lock {
        if join.is_finished() {
            return false;
        }
        return true;
    }
    true
}

fn set_callbacks(ui: &AppWindow) -> Timer {
    let (tx, rx) = channel();
    let (tx, rx) = (Arc::new(tx), Arc::new(Mutex::from(rx)));
    let join = Arc::new(Mutex::new(None));
    let timer = set_timer(&ui, &join);
    let weak = ui.as_weak();
    ui.window().on_close_requested( move || -> CloseRequestResponse {
        let ui = weak.unwrap(); 
        if let Err(error) = ui.window().hide() {
            println!("{}", error);
        }
        CloseRequestResponse::KeepWindowShown
    });
    let weak = ui.as_weak();
    ui.on_edited_server_address( move |text: SharedString| {
        let ui = weak.unwrap();
        ui.set_server_address(text);
        println!("{}", ui.get_server_address());
        ()
    });
    let weak = ui.as_weak();
    ui.on_edited_port( move |text: SharedString| {
        let ui = weak.unwrap();
        ui.set_port(text);
        println!("{}", ui.get_port());
        ()
    });
    let has_only_number_re = Regex::new(r"^(?:[0-9]|[^ -~])$").unwrap();
    ui.on_accept_only_numbers( move |text: SharedString| -> bool {
        if has_only_number_re.is_match(text.as_str()) {
            return true;
        }
        false
    });
    let weak = ui.as_weak();
    ui.on_clicked_start_stop_button( move || {
        let ui = weak.unwrap(); 
        let join = Arc::clone(&join);
        if !is_server_started(&join) { 
            ui.set_server_started(true);
            let join = Arc::clone(&join);
            start_server(&ui, Arc::clone(&rx), &join);
            return;
        }
        let join = Arc::clone(&join);
        stop_server(Arc::clone(&tx), &join);
    });
    return timer;
}

fn stop_server(tx: Arc<Sender<()>>, join: &Arc<Mutex<Option<std::thread::JoinHandle<()>>>>) {
    if !is_server_started(join) {
        return;
    }
    let _ = (*tx).send(());
}
