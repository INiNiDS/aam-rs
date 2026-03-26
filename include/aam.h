/* aam-rs C API
 *
 * Memory ownership rules:
 * - Handles created by `aam_new()` must be destroyed with `aam_free()`.
 * - Strings returned by `aam_find_*()` are heap-allocated by Rust and must be
 *   released with `aam_string_free()`.
 * - `aam_last_error()` returns an internal pointer owned by the handle;
 *   do not free it.
 */

#ifndef AAM_RS_AAM_H
#define AAM_RS_AAM_H

#ifdef __cplusplus
extern "C" {
#endif

typedef struct AamlHandle AamlHandle;

AamlHandle *aam_new(void);
void aam_free(AamlHandle *handle);

int aam_parse(AamlHandle *handle, const char *content);
int aam_load(AamlHandle *handle, const char *path);
int aam_merge(AamlHandle *handle, const char *content);
int aam_recover_simple(AamlHandle *handle, const char *content);

char *aam_find_obj(const AamlHandle *handle, const char *key);
char *aam_find_key(const AamlHandle *handle, const char *value);
char *aam_find_deep(const AamlHandle *handle, const char *key);

void aam_string_free(char *s);
const char *aam_last_error(const AamlHandle *handle);

#ifdef __cplusplus
}
#endif

#endif

