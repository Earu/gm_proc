struct WindowHandleData {
    process_id: u32,
    window_handle: winapi::HWND,
}

unsafe fn is_main_window(handle: winapi::HWND) -> bool
{
    user32::GetWindow(handle, winapi::GW_OWNER) == 0 as winapi::HWND && user32::IsWindowVisible(handle) != winapi::FALSE
}

unsafe extern "system" fn enum_windows_callback(handle: winapi::HWND, param: winapi::LPARAM) -> winapi::BOOL {
    let mut data = &mut *(param as usize as *mut WindowHandleData);
    let mut pid: winapi::DWORD = 0;

    user32::GetWindowThreadProcessId(handle, &mut pid as winapi::LPDWORD);
    if data.process_id != pid || !is_main_window(handle) {
        return winapi::TRUE;
    }

    data.window_handle = handle;
    return winapi::FALSE;
}

pub unsafe fn find_main_window(pid: winapi::DWORD) -> Option<winapi::HWND> {
    let mut handle = WindowHandleData {
        process_id: pid,
        window_handle: 0 as winapi::HWND,
    };

    let ptr = &mut handle as *mut _ as _;
    user32::EnumWindows(Some(enum_windows_callback), ptr);

    return if handle.window_handle != 0 as winapi::HWND {
		Some(handle.window_handle)
	} else {
		None
	};
}

pub unsafe fn bring_to_front(hwnd: winapi::HWND) -> bool {
	let win_pos = user32::SetWindowPos(hwnd, winapi::HWND_TOP, 0, 0, 0, 0, winapi::SWP_NOMOVE | winapi::SWP_NOSIZE);
	let win_show = user32::ShowWindow(hwnd, winapi::SW_NORMAL); // make sure to de-minimize the window

	if (win_pos == winapi::FALSE) && (win_show == winapi::FALSE) {
		return false;
	}

	true
}

pub unsafe fn bring_to_back(hwnd: winapi::HWND) -> bool {
	user32::SetWindowPos(hwnd, winapi::HWND_BOTTOM, 0, 0, 0, 0, winapi::SWP_NOMOVE | winapi::SWP_NOSIZE);
	true
}
