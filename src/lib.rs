#![feature(c_unwind)]
#![feature(exclusive_range_pattern)]

use std::borrow::Cow;
use chad_cell::ChadCell;
use sysinfo::{SystemExt, ProcessExt};

mod chad_cell; // 😎

#[cfg(target_os = "unix")]
mod unix;

#[cfg(target_os = "windows")]
mod windows;

#[macro_use] extern crate gmod;

static mut PROCESSES: ChadCell<Vec<u32>> = ChadCell::new(Vec::new());

#[cfg(target_os = "unix")]
fn spawn_process(path: &str, params: Option<Cow<'_, str>>, working_directory: Option<Cow<'_, str>>) -> Result<u32, std::io::Error> {
    unix::spawn_process(path, params, working_directory)
}

#[cfg(target_os = "windows")]
unsafe fn spawn_process(path: &str, params: Option<Cow<'_, str>>, working_directory: Option<Cow<'_, str>>) -> Result<u32, std::io::Error> {
    windows::spawn_process(path, params, working_directory)
}

#[lua_function]
unsafe fn start_process(lua: gmod::lua::State) -> i32 {
    let top = lua.get_top();
    let path: Option<Cow<'_, str>>;
    let params: Option<Cow<'_, str>>;
    let working_directory: Option<Cow<'_, str>>;

    match top {
        top if top >= 3 => {
            path = Some(lua.check_string(-3));
            params = Some(lua.check_string(-2));
            working_directory = Some(lua.check_string(-1));
        }
        2 => {
            path = Some(lua.check_string(-2));
            params = Some(lua.check_string(-1));
            working_directory = None;
        }
        _ => {
            path = Some(lua.check_string(-1));
            params = None;
            working_directory = None;
        }
    }

    let res = spawn_process(path.unwrap().as_ref(), params, working_directory);
    match res {
        Ok(pid) => {
            PROCESSES.get_mut().push(pid);

            lua.push_boolean(true);
            lua.push_number(pid as f64);
        }
        Err(_e) => {
            lua.push_boolean(false);
            lua.push_integer(0);
        }
    }

    2
}

fn signal_from_int(signal: i32) -> sysinfo::Signal {
    match signal {
        1..31 => unsafe { std::mem::transmute(signal) },
        _ => sysinfo::Signal::Kill,
    }
}

#[lua_function]
unsafe fn terminate_process(lua: gmod::lua::State) -> i32 {
    let exit_code;
    let pid;
    match lua.get_top() {
        top if top >= 2 => {
            exit_code = lua.check_integer(-1);
            pid = lua.check_integer(-2);
        }
        _ => {
            exit_code = 9;
            pid = lua.check_integer(-1);
        }
    };

    let system = sysinfo::System::new_with_specifics(sysinfo::RefreshKind::new().with_processes());
    match system.get_process(pid as usize) {
        Some(proc) => {
            let signal = signal_from_int(exit_code as i32);
            let success = proc.kill(signal);
            lua.push_boolean(success);
        }
        None => lua.push_boolean(false),
    }

    1
}

#[lua_function]
unsafe fn find_process_pids(lua: gmod::lua::State) -> i32 {
    let process_name = lua.check_string(-1);
    let system = sysinfo::System::new_with_specifics(sysinfo::RefreshKind::new().with_processes());
    let pids = system.get_processes()
        .iter()
        .filter(|p| p.1.name().contains(process_name.as_ref()))
        .map(|p| p.1.pid());

    lua.new_table();
    for (i, pid) in pids.enumerate()  {
        lua.push_integer(i as isize);
        lua.push_number(pid as f64);
        lua.set_table(-3);
    }

    1
}

#[lua_function]
unsafe fn is_process_running(lua: gmod::lua::State) -> i32 {
    let pid = lua.check_integer(-1);
    let system = sysinfo::System::new_with_specifics(sysinfo::RefreshKind::new().with_processes());
    match system.get_process(pid as usize) {
        Some(_proc) => lua.push_boolean(true),
        None => lua.push_boolean(false),
    }

    1
}

#[lua_function]
unsafe fn get_running_process_pids(lua: gmod::lua::State) -> i32 {
    lua.new_table();

    let system = sysinfo::System::new_with_specifics(sysinfo::RefreshKind::new().with_processes());
    for pid in PROCESSES.get_mut().iter_mut() {
        match system.get_process(*pid as usize) {
            Some(proc) => {
                let process_name = proc.name();

                lua.push_string(process_name);
                lua.push_integer(*pid as isize);
                lua.set_table(-3);
            }
            None => continue,
        }
    }

    1
}

#[lua_function]
unsafe fn is_process_gmod_child(lua: gmod::lua::State) -> i32 {
    let pid = lua.check_integer(-1);
    let system = sysinfo::System::new_with_specifics(sysinfo::RefreshKind::new().with_processes());
    let gmod_pid = match sysinfo::get_current_pid() {
        Ok(pid) => pid,
        Err(_err) => {
            lua.push_boolean(false);
            return 1;
        }
    };

    match system.get_process(pid as usize) {
        Some(proc) => {
            match proc.parent() {
                None => lua.push_boolean(false),
                Some(parend_pid) => lua.push_boolean(parend_pid == gmod_pid),
            }
        }
        None => lua.push_boolean(false),
    }

    1
}

#[lua_function]
unsafe fn get_gmod_pid(lua: gmod::lua::State) -> i32 {
    match sysinfo::get_current_pid() {
        Ok(pid) => lua.push_number(pid as f64),
        Err(_err) =>  lua.push_integer(-1),
    }

    1
}

#[cfg(target_os = "windows")]
#[lua_function]
unsafe fn bring_process_to_front(lua: gmod::lua::State) -> i32 {
    let pid = lua.check_integer(-1);
    match windows::find_main_window(pid as u32) {
        Some(hwnd) =>  lua.push_boolean(windows::bring_to_front(hwnd)),
        None => lua.push_boolean(false),
    }

    1
}

#[cfg(target_os = "unix")]
#[lua_function]
unsafe fn bring_process_to_front(lua: gmod::lua::State) -> i32 {
    // nothing on unix unless someone implements it
    0
}

#[cfg(target_os = "windows")]
#[lua_function]
unsafe fn bring_process_to_back(lua: gmod::lua::State) -> i32 {
    let pid = lua.check_integer(-1);
    match windows::find_main_window(pid as u32) {
        Some(hwnd) => lua.push_boolean(windows::bring_to_back(hwnd)),
        None => lua.push_boolean(false),
    }

    1
}

#[cfg(target_os = "unix")]
#[lua_function]
unsafe fn bring_process_to_back(lua: gmod::lua::State) -> i32 {
    // nothing on unix unless someone implements it
    0
}

#[gmod13_open]
unsafe fn gmod13_open(lua: gmod::lua::State) -> i32 {
    lua.new_table();

    lua.push_function(start_process);
    lua.set_field(-2, lua_string!("Start"));

    lua.push_function(terminate_process);
    lua.set_field(-2, lua_string!("Terminate"));

    lua.push_function(is_process_running);
    lua.set_field(-2, lua_string!("IsRunning"));

    lua.push_function(find_process_pids);
    lua.set_field(-2, lua_string!("FindPIDs"));

    lua.push_function(is_process_gmod_child);
    lua.set_field(-2, lua_string!("IsFromGmod"));

    lua.push_function(get_gmod_pid);
    lua.set_field(-2, lua_string!("GetGmodPID"));

    lua.push_function(get_running_process_pids);
    lua.set_field(-2, lua_string!("GetRunningPIDs"));

    lua.push_function(bring_process_to_front);
    lua.set_field(-2, lua_string!("BringToFront"));

    lua.push_function(bring_process_to_back);
    lua.set_field(-2, lua_string!("BringToBack"));

    // this takes the last value on the stack apparently, so this should work (?)
    lua.set_global(lua_string!("Process"));

    0
}

#[gmod13_close]
unsafe fn gmod13_close(_: gmod::lua::State) -> i32 {
    let system = sysinfo::System::new_with_specifics(sysinfo::RefreshKind::new().with_processes());
    for pid in PROCESSES.get_mut().iter_mut() {
        match system.get_process(*pid as usize) {
            Some(proc) => { proc.kill(sysinfo::Signal::Kill); },
            None => continue,
        }
    }

    0
}