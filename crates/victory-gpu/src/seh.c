#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#include <windows.h>

typedef HRESULT (*victory_seh_fn)(void *ctx);

/* Catches driver shader-compiler hardware exceptions (e.g. amdxc64.dll null
   dereference during D3D12 PSO creation) so a single problematic shader
   refuses only that pipeline instead of terminating the process. */
HRESULT victory_seh_call(victory_seh_fn fn, void *ctx, DWORD *caught_code) {
    DWORD code = 0;
    __try {
        return fn(ctx);
    } __except ((code = GetExceptionCode()) == EXCEPTION_ACCESS_VIOLATION ||
                code == EXCEPTION_INT_DIVIDE_BY_ZERO ||
                code == EXCEPTION_ILLEGAL_INSTRUCTION
                    ? EXCEPTION_EXECUTE_HANDLER
                    : EXCEPTION_CONTINUE_SEARCH) {
        if (caught_code) {
            *caught_code = code;
        }
        return (HRESULT)0x80004005L; /* E_FAIL */
    }
}

/* Test helper: triggers a genuine read access violation at 0x8 (matching
   amdxc64.dll+0x85fd0f) in C without invoking Rust undefined behavior. */
int victory_seh_test_fault(void) {
    volatile int *p = (volatile int *)(uintptr_t)0x8;
    return *p;
}
