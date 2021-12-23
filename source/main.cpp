#include <GarrysMod/Lua/Interface.h>

#ifdef _WIN32
#include <windows.h>
#endif

LUA_FUNCTION(StartProcess) 
{
    const bool workingDirSpecified = LUA->Top() >= 3;
    const char* path = LUA->CheckString();
    const char* params = LUA->CheckString();
    const char* workingDirectory = workingDirSpecified ? LUA->CheckString() : NULL;

#ifdef _WIN32
    INT_PTR ret = (INT_PTR)ShellExecuteA(0, "open", path, params, workingDirectory, SW_SHOW);
    if (ret > 32) {
        LUA->PushBool(true);
        return 1;
    }
    else
    {
        LUA->PushBool(false);
        LUA->PushNumber(ret);
        return 2;
    }
#endif

    // linux & macos
#if defined(__APPLE__) || defined(__linux__)
    LUA->ThrowError("OS is not compatible, gm_proc only works on Windows");
    return 0;
#endif
}

GMOD_MODULE_OPEN() 
{
    LUA->PushSpecial(GarrysMod::Lua::SPECIAL_GLOB);

    LUA->GetField(-1, "MENU_DLL");
    bool isMenuState = LUA->GetBool(-1);
    if (!isMenuState) {
        LUA->ThrowError("This module should be used in the menu state");
        return 0;
    }

    LUA->Pop(); // remove the boolean on the stack

    LUA->PushCFunction(StartProcess);
    LUA->SetField(-2, "StartProcess");
    LUA->Pop();

	return 0;
}

GMOD_MODULE_CLOSE() 
{
    return 0;
}