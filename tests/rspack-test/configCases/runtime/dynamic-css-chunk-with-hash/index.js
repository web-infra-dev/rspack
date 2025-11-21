it("load dynamic css chunk with hash", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	import("./dynamic").then(module => {
		expect(module.value).toBe("dynamic");
		// test is only for css loading
		if (__webpack_require__.f.css) {
			expect(document.getElementsByTagName("link").length).toBe(1);
		}
		import("./common").then(module => {
			expect(module.value).toBe("common");
			// test is only for css loading
			if (__webpack_require__.f.css) {
				expect(document.getElementsByTagName("link").length).toBe(1);
			}
			done();
		});
	});
}));
