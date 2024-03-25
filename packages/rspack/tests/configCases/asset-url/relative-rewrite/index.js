it("should compile", () => {
	// See: https://github.com/web-infra-dev/rspack/pull/5397
	const url = new URL("./index.css?query=yes#fragment", import.meta.url).href;
	expect(url).toBeDefined();
});
