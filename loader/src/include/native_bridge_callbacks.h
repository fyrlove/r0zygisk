#pragma once

#include <android/api-level.h>
#include <cstdint>

using NativeBridgeRuntimeCallbacks = void;
using NativeBridgeNamespace = void;
using NativeBridgeSignalHandlerFn = void (*)();

struct NativeBridgeCallbacks {
    uint32_t version;
    bool (*initialize)(const NativeBridgeRuntimeCallbacks*, const char*, const char*);
    void* (*loadLibrary)(const char*, int);
    void* (*getTrampoline)(void*, const char*, const char*, uint32_t);
    bool (*isSupported)(const char*);
    const char* (*getAppEnv)(const char*);
    bool (*isCompatibleWith)(uint32_t);
    NativeBridgeSignalHandlerFn (*getSignalHandler)(int);
    int (*unloadLibrary)(void*);
    const char* (*getError)();
    bool (*isPathSupported)(const char*);
    bool (*initAnonymousNamespace)(const char*, const char*);
    NativeBridgeNamespace* (*createNamespace)(const char*, const char*, const char*, uint64_t,
                                              const char*, NativeBridgeNamespace*);
    bool (*linkNamespaces)(NativeBridgeNamespace*, NativeBridgeNamespace*, const char*);
    void* (*loadLibraryExt)(const char*, int, const void*);
    NativeBridgeNamespace* (*getVendorNamespace)();
    NativeBridgeNamespace* (*getExportedNamespace)(const char*);
    void (*preZygoteFork)();
};
