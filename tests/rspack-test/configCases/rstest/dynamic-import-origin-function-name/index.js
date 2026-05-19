const fs = require('fs');
const path = require('path');

const sourceFile = path.resolve(
	__dirname,
	'../../../../configCases/rstest/dynamic-import-origin-function-name/src/index.js',
);

it('rewrites dynamic imports with `injectDynamicImportOrigin.functionName` override', () => {
	const content = fs.readFileSync(
		path.resolve(__dirname, 'dynamicImportOrigin.mjs'),
		'utf-8',
	);

	const importFn = 'globalThis.__custom_import__';
	const originLiteral = JSON.stringify(sourceFile);

	// Override callee from `injectDynamicImportOrigin: { functionName }` must
	// be used. `output.importFunctionName` is left at its default `'import'`
	// in this fixture — that default is normalized to "feature off", so a
	// successful rewrite here can only have come from the override path.
	expect(content).toContain(`${importFn}(\`./translations/`);
	expect(content).toContain(
		`/strings.json\`, void 0, ${originLiteral})`,
	);

	// Native `import(\`./translations/...\`, ...)` must not survive — would
	// indicate the override didn't reach the rewrite (regression guard for
	// the N-API conversion + apply-time normalization path).
	expect(content).not.toMatch(
		/\bimport\(`\.\/translations[^)]*,\s*void 0,/,
	);
});
