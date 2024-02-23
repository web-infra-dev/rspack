const fs = require("fs");
const path = require("path");
/*! Legal Comment */

/**
 * @preserve Copyright 2009 SomeThirdParty.
 * Here is the full license text and copyright
 * notice for this file. Note that the notice can span several
 * lines and is only terminated by the closing star and slash:
 */

/**
 * Utility functions for the foo package.
 * @license Apache-2.0
 */

/*! Legal Foo */

// Foo

/*
 Foo Bar
 */

// @license

/*
 * Foo
 */
// @lic

function getComments(content, regex) {
	const comments = [];
	while ((array1 = regex.exec(content)) !== null) {
		comments.push(array1[1]);
	}
	return comments;
}
it("should minify and extract comments", () => {
	const content = fs.readFileSync(
		path.resolve(__dirname, "bundle0.js.LICENSE.txt"),
		"utf-8"
	);
	expect(getComments(content, /@preserve/g).length).toBe(1);
	expect(getComments(content, /@license/g).length).toBe(2);
});
