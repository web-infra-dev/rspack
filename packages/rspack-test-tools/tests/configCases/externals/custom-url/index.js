import "./index.css";
import customJs from "custom://js";
const fs = require("fs");
const path = require("path");

it("should external custom url", function () {
	expect(customJs).toBe("custom://js");

	const css = fs.readFileSync(path.resolve(__dirname, "bundle0.css"), "utf-8");
	const importUrl = /@import url\("(.*)"\);/.exec(css)[1];
	const aUrl = /a: url\((.*)\);/.exec(css)[1];
	expect(importUrl).toBe("custom://css");
	expect(aUrl).toBe("custom://css");
});
