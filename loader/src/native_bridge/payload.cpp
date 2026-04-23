#include <sys/resource.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <unistd.h>

extern "C" [[gnu::visibility("default")]]
char daemon_addr[0x74] = {};

extern "C" [[gnu::visibility("default")]]
int my_execve(const char *pathname, char *const argv[], char *const envp[]) {
    return execve(pathname, argv, envp);
}

extern "C" [[gnu::visibility("default")]]
pid_t my_wait4(pid_t pid, int *status, int options, struct rusage *usage) {
    return wait4(pid, status, options, usage);
}
