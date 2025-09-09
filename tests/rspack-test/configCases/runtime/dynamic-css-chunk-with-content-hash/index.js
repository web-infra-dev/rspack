it("load dynamic css chunk with content hash", function (done) {
	import("./dynamic").then(module => {
		expect(module.value).toBe("dynamic");
		// test is only for css loading
		if (__webpack_require__.f.css) {
			expect(document.getElementsByTagName("link").length).toBe(1);
		}
		done();
	});
});
