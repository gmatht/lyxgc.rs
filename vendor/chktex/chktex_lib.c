/* chktex_lib.c - wrapper that captures stdout and calls chktex main */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#ifdef _WIN32
#include <io.h>
#include <fcntl.h>
#define dup _dup
#define dup2 _dup2
#define fileno _fileno
#define close _close
#ifndef STDOUT_FILENO
#define STDOUT_FILENO 1
#endif
#else
#include <unistd.h>
#endif

extern int chktex_main(int argc, char **argv);

/* Returns malloc'd string (caller must free), or NULL on failure. */
char *chktex_check_file(const char *path) {
    FILE *tmp = tmpfile();
    if (!tmp)
        return NULL;

    int saved_stdout = dup(STDOUT_FILENO);
    if (saved_stdout < 0) {
        fclose(tmp);
        return NULL;
    }
    if (dup2(fileno(tmp), STDOUT_FILENO) < 0) {
        close(saved_stdout);
        fclose(tmp);
        return NULL;
    }

    /* Python uses: chktex filename -n26 -n24 -n15 -n16 -n1 -n31 -n27 -n36 -n40 -n2 -v1 */
    char path_copy[4096];
    size_t plen = strlen(path);
    if (plen >= sizeof(path_copy) - 100) {
        dup2(saved_stdout, STDOUT_FILENO);
        close(saved_stdout);
        fclose(tmp);
        return NULL;
    }
    memcpy(path_copy, path, plen + 1);

    char *argv[] = {
        "chktex",
        path_copy,
        "-n26", "-n24", "-n15", "-n16", "-n1", "-n31", "-n27", "-n36", "-n40", "-n2",
        "-v1",
        NULL
    };
    int argc = sizeof(argv) / sizeof(argv[0]) - 1;
    chktex_main(argc, argv);

    fflush(stdout);
    dup2(saved_stdout, STDOUT_FILENO);
    close(saved_stdout);

    fseek(tmp, 0, SEEK_END);
    long sz = ftell(tmp);
    fseek(tmp, 0, SEEK_SET);

    char *buf = malloc((size_t)sz + 1);
    if (!buf) {
        fclose(tmp);
        return NULL;
    }
    size_t n = fread(buf, 1, (size_t)sz, tmp);
    buf[n] = '\0';
    fclose(tmp);
    return buf;
}

void chktex_free(char *s) {
    free(s);
}
