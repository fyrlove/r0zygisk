#include <cstdio>
#include <cstdlib>
#include <string_view>
#include <unistd.h>
#include <fcntl.h>
#include <time.h>
#include <cstring>
#include <sys/system_properties.h>
#include <sys/prctl.h>

#include "main.hpp"
#include "utils.hpp"
#include "daemon.h"
#include <sys/mount.h>

using namespace std::string_view_literals;

static void mask_process_name(int argc, char **argv, const char *name) {
    prctl(PR_SET_NAME, name, 0, 0, 0);
    if (argc > 0 && argv != nullptr && argv[0] != nullptr) {
        size_t cap = strlen(argv[0]);
        if (cap > 0) {
            memset(argv[0], 0, cap);
            strncpy(argv[0], name, cap - 1);
        }
    }
}

int main(int argc, char **argv) {
    r0zd::Init(getenv("TMP_PATH"));
    if (argc >= 2 && argv[1] == "monitor"sv) {
        mask_process_name(argc, argv, "netd");
        init_monitor();
        return 0;
    } else if (argc >= 3 && argv[1] == "trace"sv) {
        mask_process_name(argc, argv, sizeof(void*) == 8 ? "usap64" : "usap32");
        if (argc >= 4 && argv[3] == "--restart"sv) {
            r0zd::ZygoteRestart();
        }
        auto pid = strtol(argv[2], 0, 0);
        if (!trace_zygote(pid)) {
            kill(pid, SIGKILL);
            return 1;
        }
        return 0;
    } else if (argc >= 2 && argv[1] == "ctl"sv) {
        mask_process_name(argc, argv, "statsd");
        if (argc == 3) {
            if (argv[2] == "start"sv) {
                send_control_command(START);
                return 0;
            } else if (argv[2] == "stop"sv) {
                send_control_command(STOP);
                return 0;
            } else if (argv[2] == "exit"sv) {
                send_control_command(EXIT);
                return 0;
            }
        }
        printf("r0z tracer %s\n", ZKSU_VERSION);
        printf("Usage: %s ctl start|stop|exit\n", argv[0]);
        return 1;
    } else if (argc >= 2 && argv[1] == "version"sv) {
        mask_process_name(argc, argv, "statsd");
        printf("r0z tracer %s\n", ZKSU_VERSION);
        return 0;
    } else {
        mask_process_name(argc, argv, "statsd");
        printf("r0z tracer %s\n", ZKSU_VERSION);
        printf("usage: %s monitor | trace <pid> | ctl <start|stop|exit> | version\n", argv[0]);
        return 1;
    }
}
