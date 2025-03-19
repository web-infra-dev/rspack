it("should run", async () => {
	import("./a.js");
	const script = document.head._children[0];
	expect(script.getAttribute("data-create-script-injected")).toBe("true");
});
