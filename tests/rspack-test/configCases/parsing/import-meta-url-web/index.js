it("should not be replaced by file path", function() {
	const url = import.meta.url;
	expect(url).not.toMatch(/^file:\/\//);
	expect(url).toMatch(/^http:\/\//); // Or whatever the test runner uses
});
