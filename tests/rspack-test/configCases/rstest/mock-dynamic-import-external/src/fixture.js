// Codegen target (NOT executed) for the #1327/#1328 regression.
//
// `rs.mock` of an externalized specifier emits `external module "X"` for the
// hoisted mock dependency, while a dynamic `import("X")` emits a DISTINCT
// `external import "X"` — two different module ids (e.g. `X?2a28` vs `X?2d83`).
// The fix routes the EXTERNAL dynamic import through `rstest_dynamic_require`
// keyed on the clean request literal so it still resolves to the mock, while
// leaving INTERNAL dynamic imports byte-identical to upstream.
import { rs } from '@rstest/core';

rs.mock('node:child_process', () => ({ execSync: () => 'MOCKED', __mock: true }));
rs.doMock('../src/foo', () => ({ value: 'MOCKED_RELATIVE' }));

// External, mocked, dynamic-only (no static import of the same request) -> split.
export async function importMocked() {
  return import('node:child_process');
}

// External whose request literal itself contains `?` -> the request arg must be
// emitted as a clean json literal, never parsed/split on `?`.
export async function importWeird() {
  return import('node:child_process?weird');
}

// External, unmocked -> still routed through the shim (pass-through at runtime).
export async function importUnmocked() {
  return import('node:os');
}

// A non-literal importMock is rewritten with the source origin and routed
// through the request-keyed mock registry.
export async function importMockedVariable() {
  const request = 'node:child_process';
  return rs.importMock(request);
}

export async function importMockedWithImportedRs() {
  return rs.importMock('node:child_process');
}

export async function importMockedVariableWithImportedRs() {
  const request = 'node:child_process';
  return rs.importMock(request);
}

export async function importCjsManualMock() {
  return rs.importMock('node:os');
}

export function requireMockedVariable() {
  const request = 'node:child_process';
  return rs.requireMock(request);
}

export async function importRelativeMockedVariable() {
  const request = '../src/foo';
  return rs.importMock(request);
}

// Internal -> the gate must leave this as a bare `__webpack_require__.bind`.
export async function importInternal() {
  return import('./internal.js');
}
