const fs = require("fs");
const path = require("path");

it("html meta", () => {
	const htmlPath = path.join(__dirname, "./index.html");
	const htmlContent = fs.readFileSync(htmlPath, "utf-8");
	console.log(htmlContent);
	expect(
		htmlContent.includes(
			'<meta content="width=device-width, initial-scale=1, shrink-to-fit=no" name="viewport" />'
		)
	).toBe(true);
});
