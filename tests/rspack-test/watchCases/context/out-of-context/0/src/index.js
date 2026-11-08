require("./foo.less");

const fs = require("fs");
const path = require("path");

it("should detect changes for loader fileDependency which out of context", function () {
	const less = fs.readFileSync(path.resolve(__dirname, "bundle.css"), "utf-8");
	const step = /step: (.*);/.exec(less)[1];
	expect(step).toBe(WATCH_STEP);
});
