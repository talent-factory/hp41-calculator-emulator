// Phase 26 Plan 04 Task 3 — Vitest global setup.
//
// Sets `IS_REACT_ACT_ENVIRONMENT = true` so React 19's act(...) warnings
// don't flood the test output when @testing-library/react re-renders
// trigger state updates outside the test's own act() wrapping (e.g.
// debounce timers in setPressedKey).
//
// See https://react.dev/blog/2024/12/05/react-19#new-feature-batching-of-tests
// and https://testing-library.com/docs/react-testing-library/migrate-from-enzyme/#react-19

// @ts-expect-error — IS_REACT_ACT_ENVIRONMENT is not in the TS lib types.
globalThis.IS_REACT_ACT_ENVIRONMENT = true;
