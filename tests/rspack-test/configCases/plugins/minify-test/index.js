const fs = require("fs");
const path = require("path");

it("[minify-test]: chunk a should not be minified", () => {
	const content = fs.readFileSync(path.resolve(__dirname, "a.js"), "utf-8");
	expect(content).toMatch("\n");
});

it("[minify-test]: chunk b should not be minified", () => {
	const content = fs.readFileSync(path.resolve(__dirname, "b.js"), "utf-8");
	expect(content).toMatch("\n");
});

it("[minify-test]: chunk a2 should be minified", () => {
	const content = fs.readFileSync(path.resolve(__dirname, "a2.js"), "utf-8");
	expect(content).not.toMatch("\n");
});

it("[minify-test]: chunk a2 should be minified", () => {
	const content = fs.readFileSync(path.resolve(__dirname, "a2.js"), "utf-8");
	expect(content).not.toMatch("\n");
});

it("[minify-test]: inline_script should works", () => {
	const content = fs.readFileSync(path.resolve(__dirname, "c.js"), "utf-8");
	expect(content).toContain(`"<\\/sCrIpT>"`);
});
