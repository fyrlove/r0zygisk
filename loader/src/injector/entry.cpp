#include "daemon.h"
#include "logging.h"
#include "zygisk.hpp"
#include "module.hpp"

using namespace std;

void *self_handle = nullptr;

extern "C" [[gnu::visibility("default")]]
bool zygisk_entry(void* handle, const char* path) {
    LOGI("r0z library injected, version %s", ZKSU_VERSION);
    self_handle = handle;
    r0zd::Init(path);

    if (!r0zd::PingHeartbeat()) {
        LOGE("r0zd is not running");
        return false;
    }

#ifdef NDEBUG
    logging::setfd(r0zd::RequestLogcatFd());
#endif

    LOGI("Start hooking");
    hook_functions();
    return true;
}

extern "C" [[gnu::visibility("default")]]
void entry(void* handle, const char* path) {
    (void) zygisk_entry(handle, path);
}
