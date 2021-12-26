# gm_proc
Manage (Windows) processes from Garry's Mod.

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
1) Get [premake](https://github.com/premake/premake-core/releases/download/v5.0.0-alpha14/premake-5.0.0-alpha14-linux.tar.gz) add it to your `PATH`
2) Get [garrysmod_common](https://github.com/danielga/garrysmod_common) (with `git clone https://github.com/danielga/garrysmod_common --recursive --branch=x86-64-support-sourcesdk`) and set an env var called `GARRYSMOD_COMMON` to the path of the local repo
3) Run `premake5 vs2019` in your local copy of **this** repo
4) Navigate to the project directory `cd /projects/windows/vs2019`
5) Open the .sln in Visual Studio 2019+
6) Select Release, and either x64 or x86
7) Build
