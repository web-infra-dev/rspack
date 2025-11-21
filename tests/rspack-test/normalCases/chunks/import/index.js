it("should be able to use import", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	import("./two").then(function(two) {
		expect(two).toEqual(nsObj({
			default: 2
		}));
		done();
	}).catch(function(err) {
		done(err);
	});
}));
