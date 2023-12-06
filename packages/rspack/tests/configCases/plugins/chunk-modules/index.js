const fs = require("fs");
const path = require("path");

it("chunk-modules", async () => {
	const m = await import(/* webpackChunkName: "async" */ "./async");
	expect(m.default).toBe(1);
	const data = JSON.parse(
		await fs.promises.readFile(path.join(__dirname, "data.json"), "utf-8")
	);
	expect(data.main.modules.length).toBe(3);
	expect(data.main.entryModules.length).toBe(1);
	expect(data.async.modules.length).toBe(1);
	expect(data.async.entryModules.length).toBe(0);
});
