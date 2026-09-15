const fs = require('fs');
const path = require('path');

// Regression for rstest #1327 (node builtins) / #1328 (ESM-only npm packages):
// a dynamic `import()` of a MOCKED externalized specifier resolved to the real
// module because rspack mints a different external module id for the dynamic
// import (`external import "X"`) than the one the hoisted `rs.mock` patches
// (`external module "X"`). The RstestPlugin now routes EXTERNAL dynamic imports
// through `rstest_dynamic_require`, keyed on the clean request literal, so the
// mock installed under the (different) static id is found by request.
it('routes external dynamic import() of a mocked module through rstest_dynamic_require keyed on the request', () => {
	const content = fs.readFileSync(
		path.resolve(__dirname, 'mockDynamicImport.mjs'),
		'utf-8',
	);

	// The hoisted `rs.mock` appends the clean request literal as the 3rd arg, so
	// the mock factory is also registered under the request (not just the id).
	expect(content).toMatch(
		/rstest_mock\("node:child_process[^"]*",[\s\S]*?,\s*"node:child_process"\)/,
	);

	// The external, mocked, dynamic import is routed through the shim with the
	// clean request literal as the trailing bound argument.
	expect(content).toMatch(
		/rstest_dynamic_require\.bind\([^)]*,\s*"node:child_process"\)/,
	);

	// A request literal that itself contains `?` is emitted verbatim as a json
	// literal — never parsed/split on `?` (the old base-split fragility).
	expect(content).toMatch(
		/rstest_dynamic_require\.bind\([^)]*,\s*"node:child_process\?weird"\)/,
	);

	// An UNMOCKED external dynamic import is still routed through the shim
	// (pass-through at runtime). This proves the gate is "target is external",
	// not "target is mocked".
	expect(content).toMatch(/rstest_dynamic_require\.bind\([^)]*,\s*"node:os"\)/);

	// The shim is GUARDED: `rstest_dynamic_require ? <shim> : <plain require>`, so a
	// NEWER @rspack/core codegen degrades to pre-fix behavior (plain
	// `__webpack_require__(id)`) on an OLDER @rstest/core runtime that lacks
	// rstest_dynamic_require — never throwing `undefined.bind(...)`. Without this
	// guard, every external dynamic import() (even unmocked) would crash a stale
	// runtime.
	expect(content).toMatch(
		/rstest_dynamic_require\s*\?[\s\S]*?:\s*__webpack_require__\.bind\(__webpack_require__,/,
	);

	// The INTERNAL dynamic import must remain a bare `__webpack_require__.bind`
	// (byte-identical to upstream) — internal modules never split, so the gate
	// must leave their codegen untouched.
	expect(content).toMatch(
		/\.then\(__webpack_require__\.bind\(__webpack_require__,\s*"\.\/src\/internal\.js"\)\)/,
	);
	expect(content).not.toMatch(/rstest_dynamic_require\.bind\([^)]*internal/);
});
