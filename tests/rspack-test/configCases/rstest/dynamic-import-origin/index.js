const fs = require('fs');
const path = require('path');

const sourceFile = path.resolve(
	__dirname,
	'../../../../configCases/rstest/dynamic-import-origin/src/index.js',
);

it('rewrites template-literal dynamic imports with importFunctionName + origin', () => {
	const content = fs.readFileSync(
		path.resolve(__dirname, 'dynamicImportOrigin.mjs'),
		'utf-8',
	);

	const importFn = 'import.meta.__rstest_dynamic_import__';
	const originLiteral = JSON.stringify(sourceFile);

	// Template literal call (1 arg) must be rewritten and end with
	// `, void 0, "<origin>")` so origin always lands at the third-argument
	// position the runtime expects. We use `void 0` rather than the
	// identifier `undefined` because the latter can be shadowed.
	expect(content).toContain(`${importFn}(\`./translations/`);
	expect(content).toContain(
		`/strings.json\`, void 0, ${originLiteral})`,
	);

	// The shadow-resistant placeholder must be `void 0`, not `undefined`.
	expect(content).not.toMatch(
		/__rstest_dynamic_import__\(`\.\/translations[^)]*,\s*undefined,/,
	);

	// Variable specifier with attributes — origin is the third argument,
	// inserted *after* `{ with: ... }` (no `undefined` placeholder needed).
	expect(content).toContain(
		`${importFn}(name, { with: { type: 'json' } }, ${originLiteral})`,
	);

	// No doubled importFunctionName (the rspack#13673 regression).
	expect(content).not.toContain(`${importFn}${importFn}`);

	// Static literal `import('./literal.js')` must NOT be rewritten by us — it
	// goes through rspack's default ImportDependency path and ends up as a
	// `__webpack_require__.e(...).then(...)` chain.
	expect(content).not.toContain(`${importFn}('./literal.js'`);
	expect(content).not.toContain(`${importFn}("./literal.js"`);

	// `require('./nested.js')` nested inside the dynamic import argument
	// must still be collected as a dependency. Otherwise `nested.js` would
	// not be in the bundle and accessing `.name` would throw at runtime.
	expect(content).toContain(`module.exports = { name: './literal.js' }`);

	// `/* webpackIgnore: true */ import(...)` must be left as a native
	// dynamic import. We must not rewrite the callee or append origin.
	expect(content).toContain('`./ignored/${');
	expect(content).not.toMatch(
		/__rstest_dynamic_import__\(`\.\/ignored\//,
	);
});
