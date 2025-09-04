module.exports = {
	validate(stats, error, done) {
		const chunks = stats.compilation.chunks;
		const chunkNames = Array.from(chunks).map(chunk => chunk.name || chunk.id);
		
		// Verify that a chunk named "foo" is generated
		const hasFooChunk = chunkNames.some(name => name.includes("foo"));
		
		if (!hasFooChunk) {
			return done(new Error(`Expected chunk with name containing "foo", but got: ${chunkNames.join(", ")}`));
		}
		
		done();
	}
};