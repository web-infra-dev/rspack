import disabledFields from "./disabled-fields";
import emptyOptions from "./empty-options";
const fs = require("fs");

import.meta.UNKNOWN_PROPERTY = "runtime";
const preservedUnknownProperty = import.meta.UNKNOWN_PROPERTY;
let preservedComputedAccesses = 0;
const getPreservedProperty = () => {
	preservedComputedAccesses++;
	return "url";
};
const preservedComputedUrl = import.meta[getPreservedProperty()];

it("should treat an empty importMeta object like preserve-unknown", () => {
	expect(emptyOptions.url).toBe(emptyOptions.sourceUrl);
	expect(emptyOptions.webpack).toBe(5);
	expect(emptyOptions.unknown).toBe("runtime");
	expect(emptyOptions.computedUnknown).toBe("runtime");
	expect(emptyOptions.templateUnknown).toBe("runtime");
	expect(emptyOptions.computedAccesses).toBe(1);
	expect(emptyOptions.unknownOptional).toBe("runtime".length);
	expect(emptyOptions.missingOptional).toBeUndefined();
});

it("should default to preserve-unknown for output modules", () => {
	expect(preservedUnknownProperty).toBe("runtime");
	expect(preservedComputedUrl).toBeTypeOf("string");
	expect(preservedComputedAccesses).toBe(1);
});

it("should preserve disabled import.meta fields for runtime evaluation", () => {
	expect(disabledFields.url).not.toBe(disabledFields.sourceUrl);
	expect(disabledFields.urlOptional).toBe(disabledFields.url.length);
	expect(disabledFields.webpackOptional).toBeUndefined();
	expect(disabledFields.webpack).toBeUndefined();
	expect(disabledFields.main).toBeUndefined();
	expect(disabledFields.contextType).toBe("undefined");
	expect(disabledFields.globType).toBe("undefined");
	expect(disabledFields.hotType).toBe("undefined");
	expect(disabledFields.destructuredUrl).toBe(disabledFields.url);
	expect(disabledFields.destructuredWebpack).toBeUndefined();

	const source = fs.readFileSync(__filename, "utf-8");
	const importMeta = ["import", "meta"].join(".");
	expect(source).toContain(`${importMeta}.url`);
	expect(source).toContain(`${importMeta}.webpackContext`);
	expect(source).toContain(`${importMeta}.glob`);
	expect(source).toContain(`${importMeta}.webpackHot`);

	if (typeof disabledFields.filename === "string") {
		expect(disabledFields.filename).not.toBe(disabledFields.sourceFilename);
	} else {
		expect(disabledFields.filename).toBeUndefined();
	}

	if (typeof disabledFields.dirname === "string") {
		expect(disabledFields.dirname).not.toBe(disabledFields.sourceDirname);
	} else {
		expect(disabledFields.dirname).toBeUndefined();
	}
});
