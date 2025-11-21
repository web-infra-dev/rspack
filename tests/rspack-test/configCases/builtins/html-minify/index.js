const fs = require("fs");
const path = require("path");

it("html minify", () => {
	const htmlPath = path.join(__dirname, "./index.html");
	const htmlContent = fs.readFileSync(htmlPath, "utf-8");
	expect(htmlContent.includes("</script></head><body></body></html>")).toBe(true);
});
