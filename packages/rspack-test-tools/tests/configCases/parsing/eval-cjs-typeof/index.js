const fs = require("fs");

it("should compile", () => {
	if (typeof exports !== "object" || typeof module !== "object") {
		throw new Error("wrong")
	}
	const file = fs.promises.readFile(__filename);
	expect(file).not.toContain("typeof")
});
