it("context module + dynamic import + template literal", function (done) {
	const params = "index";
	import(`./child/${params}.js`).then(module => {
		expect(module.value).toBe("dynamic");
		done();
	});
});
