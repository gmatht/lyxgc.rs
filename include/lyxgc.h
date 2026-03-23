/**
 * lyxgc - LyX/LaTeX grammar checker C API
 *
 * Compile: link against liblyxgc (cdylib) or lyxgc (staticlib)
 */

#ifndef LYXGC_H
#define LYXGC_H

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Check a .tex file. Returns malloc'd output string (LyX format), or NULL on failure.
 * Caller must free with lyxgc_free().
 *
 * @param path        Path to .tex file
 * @param lang        Language (e.g. "en", "fr") or NULL for "en"
 * @param output_format "-v0", "-v1", or "-v3", or NULL for "-v1"
 * @param run_lacheck_chktex 1 to run lacheck and chktex, 0 for rules only
 */
char *lyxgc_check_file(const char *path, const char *lang, const char *output_format,
                       int run_lacheck_chktex);

/**
 * Check LaTeX text in memory. Returns malloc'd output string, or NULL on failure.
 * Caller must free with lyxgc_free().
 *
 * @param text        LaTeX source
 * @param filename    Display name for diagnostics (e.g. "stdin" or path)
 * @param lang        Language or NULL for "en"
 * @param output_format "-v0", "-v1", "-v3", or NULL
 * @param run_lacheck_chktex Ignored (always 0) - lacheck/chktex require a file path
 */
char *lyxgc_check_text(const char *text, const char *filename, const char *lang,
                       const char *output_format, int run_lacheck_chktex);

/**
 * Free a string returned by lyxgc_check_file or lyxgc_check_text.
 */
void lyxgc_free(char *ptr);

#ifdef __cplusplus
}
#endif

#endif /* LYXGC_H */
