export * from './externals.js'
export * as externalsNs from './externals.js'
export { default } from './externals.js'
export { default as externals } from './externals.js'

it('should have exports', async () => {
	const { externals, default: defaultExports, readFile, externalsNs } = await import(/*webpackIgnore: true*/'./main.mjs');
  expect(externals).toBeDefined();
	expect(defaultExports).toBeDefined();
	expect(readFile).toBeDefined();
	expect(externalsNs).toBeDefined();
});
