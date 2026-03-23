/* strlwr compatibility for Unix (Linux/macOS) - chktex expects strlwr when HAVE_STRLWR=0 */

#include <ctype.h>
#include <string.h>

/* Convert string to lowercase in-place; returns argument (like Windows _strlwr) */
char *strlwr(char *s) {
    if (!s)
        return s;
    for (char *p = s; *p; p++)
        *p = (char)tolower((unsigned char)*p);
    return s;
}
