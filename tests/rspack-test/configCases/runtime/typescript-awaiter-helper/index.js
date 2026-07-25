const { run: runA } = require("./module-a");
const { run: runB } = require("./module-b");

it("should share TypeScript async function fallbacks", async () => {
	await expect(runA("a")).resolves.toBe("done-a");
	await expect(runB("b")).resolves.toBe("done-b");
});
