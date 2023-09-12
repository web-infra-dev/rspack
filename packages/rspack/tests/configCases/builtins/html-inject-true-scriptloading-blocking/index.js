const fs = require("fs");
const path = require("path");

it("html inject is true and scriptLoading is blocking", () => {
	const htmlPath = path.join(__dirname, "./index.html");
	const htmlContent = fs.readFileSync(htmlPath, "utf-8");
	expect(htmlContent.includes('<script src="main.js"></script></body>')).toBe(
		true
	);
});
