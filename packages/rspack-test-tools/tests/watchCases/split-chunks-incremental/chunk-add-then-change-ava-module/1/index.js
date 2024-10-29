const { checkChunkModules } = __non_webpack_require__("@rspack/test-tools");

it('should compile', async () => {
	const [v1, v2] = await Promise.all([
		import('./dyn-1').then(m => m.default),
		import('./dyn-2').then(m => m.default)
	])
	expect(v1.default).toBe('shared')
	expect(v2.default).toBe('shared')

	checkChunkModules(STATS_JSON, {
		'dyn-1': [
			'dyn-1.js',
			'm.js'
		],
		'dyn-2': [
			'dyn-2.js',
		],
		shared: [
			'shared.js',
			'm.js'
		]
	})
})
