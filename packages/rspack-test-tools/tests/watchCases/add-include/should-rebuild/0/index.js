import('./no-op.js')

it('should compile', () => {
	switch (WATCH_STEP) {
		case "0":
			// do nothing
			break;
		case "1":
			const mainChunk = __STATS__.chunks.find(c => c.id === 'main');
			const find = mainChunk.modules.find(m => m.name.includes('plugin-included'));

			expect(find).toBeDefined()
			break;
		default:
			throw new Error('unexpected update');
	}
})
