it("context module + dynamic import + binary expression", function (done) {
	let params = "index";
	import("./child/" + params + ".js").then(module => {
		expect(module.value).toBe("dynamic");
		done();
	});
});
