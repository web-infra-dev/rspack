const fs = require("fs");

it("should compile", async () => {
	if (typeof exports !== "object" || typeof module !== "object") {
		throw new Error("wrong")
	}
	const file = await fs.promises.readFile(__filename, 'utf-8');
	expect(file).not.toContain(["type", "of"].join(""))
});
