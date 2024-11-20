const { checkChunkModules } = __non_webpack_require__("@rspack/test-tools");

it('should compile', async () => {
	const v1 = await import('./dyn-1').then(m => m.default)
	expect(v1.default).toBe('shared')
})
