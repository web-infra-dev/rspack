const fs = require("fs");
const path = require("path");

it("body-index.html inject", () => {
	const htmlPath = path.join(__dirname, "./body-index.html");
	const htmlContent = fs.readFileSync(htmlPath, "utf-8");
	expect(
		htmlContent.includes('<script defer src="bundle0.js"></script></body>')
	).toBe(true);
});

it("head-index.html inject", () => {
	const htmlPath = path.join(__dirname, "./head-index.html");
	const htmlContent = fs.readFileSync(htmlPath, "utf-8");
	expect(
		htmlContent.includes('<script defer src="bundle1.js"></script></head>')
	).toBe(true);
});

it("true-blocking-index.html inject", () => {
	const htmlPath = path.join(__dirname, "./true-blocking-index.html");
	const htmlContent = fs.readFileSync(htmlPath, "utf-8");
	expect(
		htmlContent.includes('<script src="bundle2.js"></script></body>')
	).toBe(true);
});

it("true-defer-index.html inject", () => {
	const htmlPath = path.join(__dirname, "./true-defer-index.html");
	const htmlContent = fs.readFileSync(htmlPath, "utf-8");
	expect(
		htmlContent.includes('<script defer src="bundle3.js"></script></head>')
	).toBe(true);
});

it("false-index.html inject", () => {
	const htmlPath = path.join(__dirname, "./false-index.html");
	const htmlContent = fs.readFileSync(htmlPath, "utf-8");
	expect(htmlContent.includes('<script src="bundle4.js"')).toBe(false);
});
