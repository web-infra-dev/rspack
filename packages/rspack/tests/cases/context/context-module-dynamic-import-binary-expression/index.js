it("context module + dynamic import + binary expression", function (done) {
	let params = "index";
	debugger;
	import("./child/" + params + ".js").then(module => {
		expect(module.value).toBe("dynamic");
		done();
	});
});
