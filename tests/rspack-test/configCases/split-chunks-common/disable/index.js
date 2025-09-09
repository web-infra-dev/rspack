import("react-dom/server");

it("should not split chunks when optimization.splitChunks is false", async () => {
	await expect(require("fs/promises").readdir(__dirname)).resolves.toEqual(
		expect.not.arrayContaining([expect.stringMatching(/~/)])
	);
});
