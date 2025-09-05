module.exports = {
	validate(stats, error, done) {
		const chunks = stats.compilation.chunks;		
		// Verify that a chunk named "foo" is generated
		const hasFooChunk = Array.from(chunks).map(chunk => chunk.name).includes("foo");
		expect(hasFooChunk).toBe(true);
	}
};