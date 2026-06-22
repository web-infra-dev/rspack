import "./loader!./file";

it("should have the file emitted", () => {
	const result = require("./extra-file.js");
	expect(result).toBe("ok");
});
