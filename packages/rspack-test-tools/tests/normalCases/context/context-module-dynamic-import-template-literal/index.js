it("context module + dynamic import + template literal", function (done) {
	let params = "index";
	import(`./child/${params}.js`).then(module => {
		expect(module.value).toBe("dynamic");
		done();
	});
});
