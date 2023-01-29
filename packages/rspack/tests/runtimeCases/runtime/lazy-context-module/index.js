it("lazy context module", function (done) {
	let params = "index";
	import(`./child/${params}.js`).then(module => {
		expect(module.value).toBe("dynamic");
		done();
	});
});
