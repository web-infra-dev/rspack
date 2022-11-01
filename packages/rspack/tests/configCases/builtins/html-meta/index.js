const fs = require("fs");
const path = require("path");

it("html meta", () => {
	const htmlPath = path.join(__dirname, "./index.html");
	expect(
		fs
			.readFileSync(htmlPath, "utf-8")
			.includes(
				'<meta content="width=device-width, initial-scale=1, shrink-to-fit=no" name="viewport" />'
			)
	).toBe(true);
});
