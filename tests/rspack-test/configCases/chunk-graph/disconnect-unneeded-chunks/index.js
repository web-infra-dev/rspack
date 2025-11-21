import './routes.js'

it('should load routes', async () => {
	const {value} = await import(/*webpackChunkName: "routes"*/ './module.js')

	expect(await value).toBe(1)

	// foo should contain no children at all
	const chunkId = __STATS__.namedChunkGroups.routes.chunks[0];
	const chunk = __STATS__.chunks.find(c => c.id === chunkId);
	expect(chunk.children.length).toBe(0);
})
