const fs = require("fs");
const path = require("path");

/*! This comment should be extracted - it's important */

/*! SuppressStringValidation - this should NOT be extracted */

/*! StartNoStringValidationRegion - this should NOT be extracted */

/*! EndNoStringValidationRegion - this should NOT be extracted */

/** This is a JSDoc comment that should be extracted */

/** SuppressStringValidation - this should NOT be extracted either */

// Regular comment - should not be extracted

/*
 * Normal block comment - should not be extracted
 */

it("should extract comments with regex lookahead and flags", () => {
	const mainFile = fs.readFileSync(
		path.resolve(__dirname, "bundle0.js"),
		"utf-8"
	);
	
	// Check that the license file was created
	const licenseExists = fs.existsSync(
		path.resolve(__dirname, "bundle0.js.LICENSE.txt")
	);
	expect(licenseExists).toBe(true);
	
	if (licenseExists) {
		const content = fs.readFileSync(
			path.resolve(__dirname, "bundle0.js.LICENSE.txt"),
			"utf-8"
		);
		
		// Should extract the important comment
		expect(content).toContain("This comment should be extracted - it's important");
		
		// Should extract the JSDoc comment
		expect(content).toContain("This is a JSDoc comment that should be extracted");
		
		// Should NOT extract suppressed comments
		expect(content).not.toContain("SuppressStringValidation");
		expect(content).not.toContain("StartNoStringValidationRegion");
		expect(content).not.toContain("EndNoStringValidationRegion");
	}
});
