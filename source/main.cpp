#include <GarrysMod/Lua/Interface.h>

#ifdef _WIN32
#include <iostream>
#include <windows.h>
#include <tlhelp32.h>
#include <Psapi.h>
#include <vector>
#include <string>

std::wstring byteToUnicode(const char* bytes) {
    std::string str = std::string(bytes);
    int size_needed = MultiByteToWideChar(CP_UTF8, 0, &str[0], (int)str.size(), NULL, 0);
    std::wstring wstrTo(size_needed, 0);
    MultiByteToWideChar(CP_UTF8, 0, &str[0], (int)str.size(), &wstrTo[0], size_needed);
    return wstrTo;
}

BOOL TerminateProcessEx(DWORD dwProcessId, UINT uExitCode)
{
    HANDLE hProcess = OpenProcess(PROCESS_TERMINATE, FALSE, dwProcessId);
    if (hProcess == NULL)
        return FALSE;

    BOOL result = TerminateProcess(hProcess, uExitCode);

    CloseHandle(hProcess);

    return result;
}

DWORD GetParentPID(DWORD pid)
{
    HANDLE handle = NULL;
    PROCESSENTRY32 processEntry = { 0 };
    DWORD parentPid = 0;
    processEntry.dwSize = sizeof(PROCESSENTRY32);
    handle = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
    if (Process32First(handle, &processEntry))
    {
        do
        {
            if (processEntry.th32ProcessID == pid)
            {
                parentPid = processEntry.th32ParentProcessID;
                break;
            }
        } while (Process32Next(handle, &processEntry));
    }

    CloseHandle(handle);
    return parentPid;
}

int GetProcessName(DWORD pid, LPSTR fname, DWORD sz)
{
    HANDLE handle = NULL;
    int err = 0;
    handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, FALSE, pid);

    if (handle)
    {
        if (GetModuleFileNameEx(handle, NULL, fname, sz) == 0)
            err = GetLastError();

        CloseHandle(handle);
    }
    else
    {
        err = GetLastError();
    }

    return err;
}  

struct HandleData {
    unsigned long process_id;
    HWND window_handle;
};

BOOL IsMainWindow(HWND handle)
{
    return GetWindow(handle, GW_OWNER) == (HWND)0 && IsWindowVisible(handle);
}

BOOL CALLBACK EnumWindowsCallback(HWND handle, LPARAM lParam)
{
    HandleData& data = *(HandleData*)lParam;
    unsigned long process_id = 0;
    GetWindowThreadProcessId(handle, &process_id);
    if (data.process_id != process_id || !IsMainWindow(handle))
        return TRUE;

    data.window_handle = handle;
    return FALSE;
}

HWND FindMainWindow(DWORD pid)
{
    HandleData data;
    data.process_id = pid;
    data.window_handle = 0;
    EnumWindows(EnumWindowsCallback, (LPARAM)&data);
    return data.window_handle;
}
#endif

LUA_FUNCTION(BringToFront)
{
#ifdef _WIN32
    int pid = static_cast<int>(LUA->CheckNumber());
    HWND winHandle = FindMainWindow(pid);
    if (winHandle)
    {
        SetWindowPos(winHandle, HWND_TOP, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
        LUA->PushBool(true);
        return 1;
    }
#endif

    LUA->PushBool(false);
    return 1;
}

LUA_FUNCTION(BringToBack)
{
#ifdef _WIN32
    int pid = static_cast<int>(LUA->CheckNumber());
    HWND winHandle = FindMainWindow(pid);
    if (winHandle)
    {
        SetWindowPos(winHandle, HWND_BOTTOM, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
        LUA->PushBool(true);
        return 1;
    }
#endif

    LUA->PushBool(false);
    return 1;
}

LUA_FUNCTION(IsFromGmod) 
{
#ifdef _WIN32
    int pid = static_cast<int>(LUA->CheckNumber());

    DWORD curProcessPid = GetCurrentProcessId();
    char curProcName[MAX_PATH] = { 0 };
    int err = GetProcessName(curProcessPid, curProcName, MAX_PATH);
    if (err != 0) {
        LUA->PushBool(false);
        return 1;
    }

    std::string gmodName = std::string(curProcName);

    DWORD parentPid = GetParentPID(pid);
    char targetProcessName[MAX_PATH] = { 0 };
    err = GetProcessName(parentPid, targetProcessName, MAX_PATH);
    if (err != 0) {
        LUA->PushBool(false);
        return 1;
    }

    bool isGmodProc = std::string(targetProcessName).compare(gmodName);
    LUA->PushBool(isGmodProc);
    return 1;
#endif

    LUA->PushBool(false);
    return 1;
}

std::vector<int> runningPids = std::vector<int>();
LUA_FUNCTION(StartProcess) 
{
#ifdef _WIN32 
    const char* path = LUA->CheckString(-1);
    const char* params = LUA->Top() > 1 && LUA->GetType(-2) == (int)GarrysMod::Lua::Type::String ? LUA->GetString(-2) : "";
    const char* workingDirectory = LUA->Top() > 2 && LUA->GetType(3) == (int)GarrysMod::Lua::Type::String ? LUA->GetString(-3) : NULL;

    SHELLEXECUTEINFO exInfo = { sizeof(exInfo) };
    exInfo.lpVerb = "open";
    exInfo.lpFile = path;
    exInfo.lpParameters = params;
    exInfo.lpDirectory = workingDirectory;
    exInfo.nShow = SW_SHOW;
    exInfo.fMask = SEE_MASK_NOCLOSEPROCESS;
    exInfo.hwnd = NULL;
    exInfo.cbSize = sizeof exInfo;
    
    if (ShellExecuteExA(&exInfo) && exInfo.hProcess) {
        DWORD pid = GetProcessId(exInfo.hProcess);
        runningPids.push_back(pid);
        LUA->PushBool(true);
        LUA->PushNumber(pid);
        return 2;
    }
    else
    {
        LUA->PushBool(false);
        LUA->PushNumber(0);
        return 2;
    }
#endif

    LUA->PushBool(false);
    LUA->PushNumber(0);
    return 2;
}

LUA_FUNCTION(GetRunningPIDs) 
{
    LUA->CreateTable();
#ifdef _WIN32
    for (int i = 0; i < runningPids.size(); i++)
    {
        int pid = runningPids.at(i);
        char procName[MAX_PATH] = { 0 };
        int err = GetProcessName(pid, procName, MAX_PATH);
        if (err != 0) continue;

        LUA->PushNumber(pid);
        LUA->SetField(-2, procName);
    }
#endif

    return 1;
}

LUA_FUNCTION(EndProcess) 
{
#ifdef _WIN32
    const bool exitCodeSpecified = LUA->Top() >= 2;
    int pid = static_cast<int>(LUA->CheckNumber(-1));
    int exitCode = exitCodeSpecified ? static_cast<int>(LUA->CheckNumber(-1)) : 0;
    BOOL success = TerminateProcessEx(pid, exitCode);
    LUA->PushBool(success == TRUE ? true : false);
    return 1;
#endif

    LUA->PushBool(false);
    return 1;
}

LUA_FUNCTION(FindProcessPIDs)
{
#ifdef _WIN32
    const char* processName = LUA->CheckString(-1);
    std::wstring targetProcessName = byteToUnicode(processName);

    HANDLE snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0); //all processes

    PROCESSENTRY32W entry; //current process
    entry.dwSize = sizeof entry;

    LUA->CreateTable();
    if (!Process32FirstW(snap, &entry)) { //start with the first in snapshot
        return 1;
    }

    int i = 1;
    do {
        if (std::wstring(entry.szExeFile) == targetProcessName) {
            LUA->PushNumber(i);
            LUA->PushNumber(entry.th32ProcessID);
            LUA->SetTable(-3);
            i++;
        }
    } while (Process32NextW(snap, &entry)); //keep going until end of snapshot

    return 1;
#endif

    LUA->CreateTable();
    return 1;
}

LUA_FUNCTION(IsProcessRunning) 
{
#ifdef _WIN32
    int pid = static_cast<int>(LUA->CheckNumber(-1));
    HANDLE process = OpenProcess(SYNCHRONIZE, FALSE, pid);
    DWORD ret = WaitForSingleObject(process, 0);
    CloseHandle(process);

    bool isRunning = ret == WAIT_TIMEOUT;
    LUA->PushBool(isRunning);
    return 1;
#endif

    LUA->PushBool(false);
    return 1;
}

LUA_FUNCTION(GetGmodPID) {
#ifdef _WIN32
    int pid = GetCurrentProcessId();
#else
    int pid = -1;
#endif
    LUA->PushNumber(pid);
    return 1;
}

GMOD_MODULE_OPEN() 
{
    LUA->PushSpecial(GarrysMod::Lua::SPECIAL_GLOB);

        LUA->CreateTable();

            LUA->PushCFunction(StartProcess);
            LUA->SetField(-2, "Start");

            LUA->PushCFunction(EndProcess);
            LUA->SetField(-2, "Terminate");

            LUA->PushCFunction(IsProcessRunning);
            LUA->SetField(-2, "IsRunning");

            LUA->PushCFunction(FindProcessPIDs);
            LUA->SetField(-2, "FindPIDs");

            LUA->PushCFunction(IsFromGmod);
            LUA->SetField(-2, "IsFromGmod");

            LUA->PushCFunction(GetGmodPID);
            LUA->SetField(-2, "GetGmodPID");

            LUA->PushCFunction(GetRunningPIDs);
            LUA->SetField(-2, "GetRunningPIDs");

            LUA->PushCFunction(BringToFront);
            LUA->SetField(-2, "BringToFront");

            LUA->PushCFunction(BringToBack);
            LUA->SetField(-2, "BringToBack");

        LUA->SetField(-2, "Process");
        
    LUA->Pop();

	return 0;
}

GMOD_MODULE_CLOSE() 
{
#if _WIN32
    for (int i = 0; i < runningPids.size(); i++) 
    {
        int pid = runningPids.at(i);
        TerminateProcessEx(pid, 0);
    }
#endif

    return 0;
}