import("react-dom/server");

it("split modules in node_modules as vendors", async () => {
	await expect(require("fs/promises").readdir(__dirname)).resolves.toEqual(
		expect.arrayContaining([expect.stringMatching(/vendors-/)])
	);
});
