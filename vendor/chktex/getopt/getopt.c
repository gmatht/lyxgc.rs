/* Minimal getopt_long for Windows - public domain */
#include "getopt.h"
#include <stdio.h>
#include <string.h>

int optind = 1;
int opterr = 1;
int optopt;
char *optarg = NULL;

static char *place = "";

static int getopt_int(int argc, char *const argv[], const char *optstring) {
    char *oli;
    if (!*place) {
        if (optind >= argc || argv[optind][0] != '-' || !argv[optind][1])
            return -1;
        if (argv[optind][1] == '-' && !argv[optind][2]) {
            optind++;
            return -1;
        }
        place = argv[optind] + 1;
    }
    optopt = (unsigned char)*place++;
    oli = strchr(optstring, optopt);
    if (!oli || optopt == ':') {
        if (opterr && *optstring != ':')
            fprintf(stderr, "invalid option: -%c\n", optopt);
        return '?';
    }
    if (oli[1] == ':') {
        if (*place)
            optarg = place;
        else if (optind + 1 < argc)
            optarg = argv[++optind];
        else {
            if (opterr && *optstring != ':')
                fprintf(stderr, "option requires an argument: -%c\n", optopt);
            return *optstring == ':' ? ':' : '?';
        }
        place = "";
        optind++;
    }
    if (!*place)
        optind++;
    return optopt;
}

int getopt(int argc, char *const argv[], const char *optstring) {
    return getopt_int(argc, argv, optstring);
}

int getopt_long(int argc, char *const argv[], const char *optstring,
                const struct option *longopts, int *longindex) {
    int i, match = -1;
    size_t namelen;
    if (optind >= argc || !argv[optind] || argv[optind][0] != '-')
        return -1;
    if (argv[optind][1] == '-') {
        place = "";
        optind++;
        if (!argv[optind - 1][2])
            return -1;
        place = (char *)argv[optind - 1] + 2;
        namelen = strcspn(place, "=");
        for (i = 0; longopts[i].name; i++) {
            if (strlen(longopts[i].name) != namelen)
                continue;
            if (strncmp(place, longopts[i].name, namelen))
                continue;
            match = i;
            break;
        }
        if (match < 0) {
            if (opterr)
                fprintf(stderr, "unknown option: --%s\n", place);
            place = "";
            optind++;
            return '?';
        }
        if (longindex)
            *longindex = match;
        place += namelen;
        if (*place == '=') {
            optarg = place + 1;
            place = "";
        } else if (longopts[match].has_arg == required_argument) {
            if (optind < argc)
                optarg = argv[optind++];
            else {
                if (opterr)
                    fprintf(stderr, "option --%s requires an argument\n", longopts[match].name);
                place = "";
                return *optstring == ':' ? ':' : '?';
            }
        }
        place = "";
        return longopts[match].val;
    }
    return getopt_int(argc, argv, optstring);
}
