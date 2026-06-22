const fs = require('fs');
const path = require('path');

const sourceFile = path.resolve(
	__dirname,
	'../../../../configCases/rstest/require-resolve-origin/src/index.js',
);

it('rewrites require.resolve calls with source module origin', () => {
	const content = fs.readFileSync(
		path.resolve(__dirname, 'requireResolveOrigin.js'),
		'utf-8',
	);

	const helper = '__rstest_require_resolve__';
	const originLiteral = JSON.stringify(sourceFile);

	expect(content).toContain(`${helper}('./target', ${originLiteral})`);
	expect(content).toContain(`${helper}(name, ${originLiteral})`);
	expect(content).toContain(
		`${helper}('./target', { paths: [__dirname] }, ${originLiteral})`,
	);

	// Nested require inside the argument must still be collected as a dependency.
	expect(content).toContain("module.exports = { name: './target' }");

	// Nested require.resolve calls inside arguments should still be rewritten, while
	// `webpackIgnore` only affects require.resolve when commonjsMagicComments is
	// enabled, and shadowed require must not be rewritten.
	if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
		expect(content).toContain(
			`${helper}((__rspack_context.r(161)/* .name */.name), ${originLiteral})`,
		);
	} else {
		expect(content).toContain(
			`${helper}((__webpack_require__(161)/* .name */.name), ${originLiteral})`,
		);
	}
	expect(content).toContain(
		`${helper}(/* webpackIgnore: true */ './ignored', ${originLiteral})`,
	);
	expect(content).toContain("require.resolve('./shadowed')");
});

it('rewrites require.resolve calls with `functionName` override', () => {
	const content = fs.readFileSync(
		path.resolve(__dirname, 'requireResolveOriginFunctionName.js'),
		'utf-8',
	);

	const helper = 'globalThis.__custom_require_resolve__';
	const originLiteral = JSON.stringify(sourceFile);

	expect(content).toContain(`${helper}('./target', ${originLiteral})`);
	expect(content).toContain(
		`${helper}('./target', { paths: [__dirname] }, ${originLiteral})`,
	);
	expect(content).not.toContain(
		`__rstest_require_resolve__('./target', ${originLiteral})`,
	);
});

it('respects webpackIgnore when commonjsMagicComments is enabled', () => {
	const content = fs.readFileSync(
		path.resolve(__dirname, 'requireResolveOriginMagicComments.js'),
		'utf-8',
	);

	const helper = '__rstest_require_resolve__';
	const originLiteral = JSON.stringify(sourceFile);

	expect(content).toContain(`${helper}('./target', ${originLiteral})`);
	expect(content).toContain("require.resolve(/* webpackIgnore: true */ './ignored')");
	expect(content).not.toContain(`${helper}('./ignored', ${originLiteral})`);
});
