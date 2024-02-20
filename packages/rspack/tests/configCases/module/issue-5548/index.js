const fs = require("fs");

it("eager should not split chunks", async () => {
	await import("./dynamic").then(({ dynamic }) => {
		expect(dynamic).toBe("dynamic");
	});
	const files = fs.readdirSync(__dirname);
	expect(files).toStrictEqual(["main.js"]);
});
