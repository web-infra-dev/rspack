const fs = require("fs");
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

it("should contain custom banner", () => {
	const custom = fs.readFileSync(__filename, "utf-8").startsWith("/*! custom */");
	expect(custom).toBeTruthy();
});
