// This config need to be set on initial evaluation to be effective
__webpack_nonce__ = "nonce";

it("should prefetch and preload child chunks on chunk load", async () => {
	let link, script;

	expect(document.head._children).toHaveLength(1);

	// Test prefetch from entry chunk
	link = document.head._children[0];
	expect(link.getAttribute("data-prefetch-injected")).toBe("true");
	expect(link.getAttribute("data-preload-injected")).toBeFalsy();

	const promise = import(
		/* webpackChunkName: "chunk1", webpackPrefetch: true */ "./chunk1"
	);

	// Test preload of chunk1-b
	link = document.head._children[2];
	expect(link.getAttribute("data-preload-injected")).toBe("true");
	expect(link.getAttribute("data-prefetch-injected")).toBeFalsy();
});
