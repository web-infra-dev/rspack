it("should compile", () => {
	// See: https://github.com/web-infra-dev/rspack/pull/5397
	const url = new URL("./index.css?query=yes#fragment", import.meta.url).href;
	expect(url).toBeDefined();
});

it("should replace repeated static URLs after Unicode text", () => {
	const label = "资源🌍";
	const urls = [
		new URL("./index.css?query=one#first", import.meta.url).href,
		new URL("./index.css?query=two#second", import.meta.url).href,
		new URL("./index.css?query=one#first", import.meta.url).href
	];
	expect(label).toBe("资源🌍");
	expect(urls[0]).toContain("index.css?query=one#first");
	expect(urls[1]).toContain("index.css?query=two#second");
	expect(urls[2]).toBe(urls[0]);
});
