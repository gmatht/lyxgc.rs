/* lacheck_lib.c - thin wrapper that captures stdout and calls lacheck_main */

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

#include <setjmp.h>

extern jmp_buf lacheck_exit_buf;
extern int lacheck_main(int argc, char *argv[]);

/* Returns malloc'd string (caller must free), or NULL on failure. */
char *lacheck_check_file(const char *path) {
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

    char *argv[3] = {"lacheck", (char *)path, NULL};
    int code = setjmp(lacheck_exit_buf);
    if (code == 0) {
        lacheck_main(2, argv);
    }
    /* If longjmp was used, code != 0; we still read what was written */

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

void lacheck_free(char *s) {
    free(s);
}
