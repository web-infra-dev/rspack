const fs = require("fs");
const path = require("path");

it("html meta", () => {
	const htmlPath = path.join(__dirname, "./index.html");
	const htmlContent = fs.readFileSync(htmlPath, "utf-8");
	expect(htmlContent).toContain('<meta content="width=device-width,initial-scale=1,shrink-to-fit=no" name="viewport">');
	expect(htmlContent).toContain('<meta a="b" name="test">');
});
