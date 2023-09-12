const fs = require("fs");
const path = require("path");

it("html inject is head", () => {
	const htmlPath = path.join(__dirname, "./index.html");
	const htmlContent = fs.readFileSync(htmlPath, "utf-8");
	expect(
		htmlContent.includes('<script src="main.js" defer></script></head>')
	).toBe(true);
});
