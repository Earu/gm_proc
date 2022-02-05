# gm_proc
Cross-platform process management module for Garry's Mod.

### Usage
(success: **bool**, pid: **number**) `Process.Start`(path: **string**, parameters?: **string**, working_directory?: **string**)

(success: **bool**) `Process.Terminate`(pid: **number**)

(pids: **table**) `Process.FindPIDs`(exe_name: **string**)

(is_running: **bool**) `Process.IsRunning`(pid: **number**)

(pids: **table**) `Process.GetRunningPIDs`()

(from_gmod: **bool**) `Process.IsFromGmod`(pid: **number**)

#### Example
```lua
require("proc")

local cmd_pid = 0
local function is_cmd_running()
	if cmd_pid == 0 then
		local procs = Process.FindPIDs("cmd.exe")
		if #procs > 0 then
			cmd_pid = procs[1]
			return true
		else
			return false
		end
	else
		return Process.IsRunning(cmd_pid)
	end
end

if not is_cmd_running() then
	cmd_pid = Process.Start("cmd.exe")
end
```

### Compiling
- Open a terminal
- Install **cargo** if you dont have it (on Windows => https://win.rustup.rs) (on Linux/Macos => curl https://sh.rustup.rs -sSf | sh)
- Get [git](https://git-scm.com/downloads) or download the archive for the repository directly
- `git clone https://github.com/Earu/gm_zip` (ignore this if you've downloaded the archive)
- Run `cd gm_zip`
- `cargo build`
- Go in `target/debug` and rename the binary according to your branch and realm (gmsv_zip_win64.dll, gmcl_zip_win64.dll, gmsv_zip_linux.dll, gmcl_zip_linux.dll, gmcl_zip_osx64.dll)
- Put the binary in your gmod `lua/bin` directory

*Note: Even on other platforms than Windows the extension of your modules **needs** to be **.dll***
