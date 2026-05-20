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

// v2.2.1 / quick-task 260516-c1p — jsdom does not implement
// Element.scrollIntoView. App.tsx's two auto-scroll useEffects
// (printEndRef + activeStepRef) call it whenever the print log grows
// or the program counter changes, which surfaced as unhandled
// TypeError noise during the existing Phase 26 tests and as a
// hard test failure for Group G tests that seed prgm=true on the
// initial mount (the program panel renders, ref captures the div,
// scrollIntoView throws, the render tree fails to settle, and
// findKey('shift') no longer locates the SHIFT key). A no-op stub
// at Element.prototype level fixes both at once.
if (!Element.prototype.scrollIntoView) {
  Element.prototype.scrollIntoView = function scrollIntoViewStub(): void {
    /* no-op for jsdom */
  };
}
