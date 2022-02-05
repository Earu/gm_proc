use std::io::{Error, ErrorKind};

use x11::xlib::{
    XOpenDisplay,
    XDefaultRootWindow,
    XInternAtom,
    Window,
    Success,
    True,
    XQueryTree,
    _XDisplay,
    XGetWindowProperty,
    False,
    XA_CARDINAL,
    XFree,
    XRaiseWindow,
    XSetInputFocus,
    RevertToNone,
    CurrentTime,
    XCloseDisplay,
    XLowerWindow,
    RevertToPointerRoot
};

unsafe fn search_windows(display: *mut _XDisplay, pid: u64, atom_pid: u64, window: Window, results: &mut Vec<Window>) {
    // get the PID for the current Window.
    let mut typew: u64 = 0;
    let mut format: i32 = 0;
    let mut n_items: u64 = 0;
    let mut bytes_after: u64 = 0;
    let mut prop_pid: *mut u8 = 0 as *mut _;
    if XGetWindowProperty(
        display,
        window,
        atom_pid,
        0,
        1,
        False,
        XA_CARDINAL,
        &mut typew,
        &mut format,
        &mut n_items,
        &mut bytes_after,
        &mut prop_pid
    ) == Success.into() {
        if prop_pid != 0 as *mut _ {
            // if the PID matches, add this window to the result set.
            if pid == *(prop_pid as *const u64) {
                results.push(window);
            }

            XFree(prop_pid as *mut _);
        }
    }

    // Recurse into child windows.
    let mut root_window: u64 = 0;
    let mut parent_window: u64 = 0;
    let mut children: *mut u64 = 0 as *mut _;
    let mut children_count: u32 = 0;
    if XQueryTree(display, window, &mut root_window, &mut parent_window, &mut children, &mut children_count) != 0 {
        let slice = std::slice::from_raw_parts(children, children_count as usize);
        for window in slice {
            search_windows(display, pid, atom_pid, *window, results);
        }
    }
}

unsafe fn find_windows(display: *mut _XDisplay, pid: u64) -> Result<Vec<Window>, Error> {
    if display.is_null() {
        return Err(Error::new(ErrorKind::Unsupported, "Your distro is not supported, XLib not found"));
    }

    let root: Window = XDefaultRootWindow(display);
    let atom_pid = XInternAtom(display, "_NET_WM_PID\0".as_ptr() as *const i8, True);
    if atom_pid == 0 {
        return Err(Error::new(ErrorKind::InvalidData, "Cannot find XLib Atom for target process"));
    }

    let mut results: Vec<Window> = Vec::new();
    search_windows(display, pid, atom_pid, root, &mut results);

    Ok(results)
}

pub unsafe fn bring_window_to_front(pid: u64) -> Result<bool, Error> {
    let display = XOpenDisplay(0 as *const _);
    match find_windows(display, pid) {
        Ok(results) => {
            let target_count = results.len();
            let mut count = 0;
            for window in results {
                let raised = XRaiseWindow(display, window);
                let focused = XSetInputFocus(display, window, RevertToNone, CurrentTime);
                if raised == True && focused == True {
                    count += 1;
                }
            }

            XCloseDisplay(display);
            Ok(count == target_count)
        },
        Err(e) => Err(e),
    }
}

pub unsafe fn bring_process_to_back(pid: u64) -> Result<bool, Error> {
    let display = XOpenDisplay(0 as *const _);
    match find_windows(display, pid) {
        Ok(results) => {
            let target_count = results.len();
            let mut count = 0;
            for window in results {
                let lowered = XLowerWindow(display, window);
                let focused = XSetInputFocus(display, 0, RevertToPointerRoot, CurrentTime);
                if lowered == True && focused == True {
                    count += 1;
                }
            }

            XCloseDisplay(display);
            Ok(count == target_count)
        },
        Err(e) => Err(e),
    }
}