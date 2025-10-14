it("should be able the catch a incorrect import", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	var expr = "1";
	import("./folder/" + expr).then(function() {
		done(new Error("should not be called"));
	}).catch(function(err) {
		expect(err).toBeInstanceOf(Error);
		done();
	});
}));
