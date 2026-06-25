/** @type {import('fs')} */
const fs = require("fs");

import ignored from "./ignored-module";

it("should startsWith use strict", function () {
	const source = fs.readFileSync(__filename, "utf-8");
	expect(source.length).not.toBe(0);
	expect(ignored).toEqual({});
	expect(
		source.startsWith('"use strict"') || source.startsWith("'use strict'")
	).toBeTruthy();
});
