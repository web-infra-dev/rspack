it("should load css chunk", function (done) {
	import("./common").then(module => {
		// test is only for css loading
		if (__webpack_require__.f.css) {
			expect(document.getElementsByTagName("link").length).toBe(1);
		}
		done();
	});
});
