it("should parse multiple expressions in a require", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	var name = "abc";
	require(["./" + name + "/" + name + "Test"], function(x) {
		expect(x).toBe("ok");
		done();
	});
}));
