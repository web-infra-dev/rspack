const fs = require("fs");
const path = require("path");

it("html plugin should add only one runtime chunk script", () => {
	const htmlPath = path.join(__dirname, "./index.html");
	const htmlContent = fs.readFileSync(htmlPath, "utf-8");
	throw Error("failed");
	expect(htmlContent.match(/runtime\.js/g)).toHaveLength(1);
});
