it("should load css chunk", async function () {
	const module = await import("./share");
	expect(module.value).toBe(1);
	// test is only for css loading
	if (__webpack_require__.f.css) {
		expect(document.getElementsByTagName("link").length).toBe(2);
	}
});
