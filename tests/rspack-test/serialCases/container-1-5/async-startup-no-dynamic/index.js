const fs = require("fs");
const path = require("path");
import App from "./App";

it("should emit promise-based bootstrap in CommonJS bundle", () => {
	const content = fs.readFileSync(path.join(__dirname, "main.js"), "utf-8");
	expect(content).toContain("module.exports = Promise.resolve().then");
});

it("should await federation startup without dynamic import", () => {
	return App().then(rendered => {
		expect(rendered).toBe("ComponentA rendered with [This is react 1.0.0]");
	});
});
