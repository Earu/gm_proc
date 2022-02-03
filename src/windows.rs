use std::{borrow::Cow};

use winapi::{
    shared::{minwindef::{FALSE, BOOL, LPARAM, LPDWORD, TRUE, DWORD}, windef::HWND},
    um::{winuser::{
        GW_OWNER, HWND_TOP, SWP_NOMOVE, SWP_NOSIZE, SW_NORMAL, HWND_BOTTOM,
        IsWindowVisible, GetWindow, GetWindowThreadProcessId, EnumWindows, ShowWindow, SetWindowPos, SW_SHOW
    }, shellapi::{ShellExecuteExA, SHELLEXECUTEINFOA, SEE_MASK_NOCLOSEPROCESS}, processthreadsapi::GetProcessId}
};

struct WindowHandleData {
    process_id: u32,
    window_handle: HWND,
}

unsafe fn is_main_window(handle: HWND) -> bool
{
    GetWindow(handle, GW_OWNER) == 0 as HWND && IsWindowVisible(handle) != FALSE
}

unsafe extern "system" fn enum_windows_callback(handle: HWND, param: LPARAM) -> BOOL {
    let mut data = &mut *(param as usize as *mut WindowHandleData);
    let mut pid: DWORD = 0;

    GetWindowThreadProcessId(handle, &mut pid as LPDWORD);
    if data.process_id != pid || !is_main_window(handle) {
        return TRUE;
    }

    data.window_handle = handle;
    return FALSE;
}

pub unsafe fn find_main_window(pid: u32) -> Option<HWND> {
    let mut handle = WindowHandleData {
        process_id: pid,
        window_handle: 0 as HWND,
    };

    let ptr = &mut handle as *mut _ as _;
    EnumWindows(Some(enum_windows_callback), ptr);

    return if handle.window_handle != 0 as HWND {
		Some(handle.window_handle)
	} else {
		None
	};
}

pub unsafe fn bring_to_front(hwnd: HWND) -> bool {
	let win_pos = SetWindowPos(hwnd, HWND_TOP, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
	let win_show = ShowWindow(hwnd, SW_NORMAL); // make sure to de-minimize the window

	if (win_pos == FALSE) && (win_show == FALSE) {
		return false;
	}

	true
}

pub unsafe fn bring_to_back(hwnd: HWND) -> bool {
	SetWindowPos(hwnd, HWND_BOTTOM, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
	true
}

pub unsafe fn spawn_process(path: &str, params: Option<Cow<'_, str>>, working_directory: Option<Cow<'_, str>>) -> Result<u32, std::io::Error> {
    let mut info: SHELLEXECUTEINFOA = std::mem::zeroed();
    info.lpVerb = "open".as_ptr() as *const _;
    info.lpFile = path.as_ptr() as *const _;
    info.lpParameters = params.as_ref().map(|p| p.as_ptr() as *const _).unwrap_or(std::ptr::null());
    info.lpDirectory = working_directory.as_ref().map(|p| p.as_ptr() as *const _).unwrap_or(std::ptr::null());
    info.nShow = SW_SHOW;
    info.fMask = SEE_MASK_NOCLOSEPROCESS;
    info.hwnd = 0 as HWND;
    info.cbSize = std::mem::size_of::<SHELLEXECUTEINFOA>() as u32;

    match ShellExecuteExA(&mut info) {
        TRUE => Ok(GetProcessId(info.hProcess)),
        _ => Err(std::io::Error::last_os_error()),
    }
}