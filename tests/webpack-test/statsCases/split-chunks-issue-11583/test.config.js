module.exports = {
	validate(stats, error, actual) {
		const chunks = stats.compilation.chunks;		
		const hasFooChunk = Array.from(chunks).map(chunk => chunk.name).includes("foo");
		expect(hasFooChunk).toBe(true);
	}
};