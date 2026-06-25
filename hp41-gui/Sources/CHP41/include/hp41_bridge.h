#ifndef HP41_BRIDGE_H
#define HP41_BRIDGE_H

typedef struct Calculator Calculator;
typedef struct CancellationHandle CancellationHandle;

/* Calculator and returned string pointers are caller-owned and must be
 * released exactly once with their matching destroy/free function. Calculator
 * functions are single-threaded; only hp41_cancel may run concurrently.
 * JSON responses contain status="ok" or status="error". */
Calculator *hp41_create(const char *state_path);
void hp41_destroy(Calculator *calculator);
CancellationHandle *hp41_cancellation_handle(const Calculator *calculator);
void hp41_cancel(const CancellationHandle *handle);
void hp41_cancellation_handle_destroy(CancellationHandle *handle);
void hp41_press(Calculator *calculator, const char *key);
char *hp41_state_json(Calculator *calculator);
char *hp41_request_json(Calculator *calculator, const char *request_json);
void hp41_string_free(char *value);

#endif
