const path = require('path');

it('resolves import.meta.rstest with the source module path', () => {
	const bundlePath = path.resolve(__dirname, 'importMetaRstest.js');
	const result = eval('require')(bundlePath);

	expect(result.direct).toEqual({ source: 'entry' });
	expect(result.optional).toBeUndefined();
	expect(result.property).toBe('entry');
	expect(result.type).toBe('object');
	expect(result.branch).toBe(true);

	expect(result.imported.direct).toBeUndefined();
	expect(result.imported.optional).toBeUndefined();
	expect(result.imported.type).toBe('undefined');
	expect(result.imported.branch).toBe(false);

	expect(result.calls).toContain(
		path.resolve(
			__dirname,
			'../../../../configCases/rstest/import-meta-rstest/src/index.js',
		),
	);
	expect(result.calls).toContain(
		path.resolve(
			__dirname,
			'../../../../configCases/rstest/import-meta-rstest/src/imported.js',
		),
	);
});

it('returns undefined when the runtime resolver is unavailable', () => {
	const resolver = globalThis['@rstest/core/import-meta'];
	delete globalThis['@rstest/core/import-meta'];
	try {
		const bundlePath = path.resolve(__dirname, 'withoutResolver.js');
		const result = eval('require')(bundlePath);

		expect(result.direct).toBeUndefined();
		expect(result.optional).toBeUndefined();
		expect(result.type).toBe('undefined');
		expect(result.branch).toBe(false);
	} finally {
		globalThis['@rstest/core/import-meta'] = resolver;
	}
});
