type Matcher = string | RegExp | (string | RegExp)[];

/**
 * Returns a function that returns the string with the token replaced with the replacement
 * @example
 * ```js
 * const test = asRegExp("test");
 * test.test("test"); // true
 *
 * const test2 = asRegExp(/test/);
 * test2.test("test"); // true
 * ```
 */
export const asRegExp = (test: string | RegExp): RegExp => {
	if (typeof test === "string") {
		// Escape special characters in the string to prevent them from being interpreted as special characters in a regular expression. Do this by
		// adding a backslash before each special character
		return new RegExp("^" + test.replace(/[-[\]{}()*+?.,\\^$|#\s]/g, "\\$&"));
	}
	return test;
};

export const matchPart = (str: string, test: Matcher) => {
	if (!test) return true;

	if (Array.isArray(test)) {
		return test.map(asRegExp).some(regExp => regExp.test(str));
	} else {
		return asRegExp(test).test(str);
	}
};

interface MatchObject {
	test?: Matcher;
	include?: Matcher;
	exclude?: Matcher;
}

/**
 * Tests if a string matches a match object. The match object can have the following properties:
 * - `test`: a RegExp or an array of RegExp
 * - `include`: a RegExp or an array of RegExp
 * - `exclude`: a RegExp or an array of RegExp
 *
 * The `test` property is tested first, then `include` and then `exclude`.
 *
 * @example
 * ```js
 * ModuleFilenameHelpers.matchObject({ test: "foo.js" }, "foo.js"); // true
 * ModuleFilenameHelpers.matchObject({ test: /^foo/ }, "foo.js"); // true
 * ModuleFilenameHelpers.matchObject({ test: [/^foo/, "bar"] }, "foo.js"); // true
 * ModuleFilenameHelpers.matchObject({ test: [/^foo/, "bar"] }, "baz.js"); // false
 * ModuleFilenameHelpers.matchObject({ include: "foo.js" }, "foo.js"); // true
 * ModuleFilenameHelpers.matchObject({ include: "foo.js" }, "bar.js"); // false
 * ModuleFilenameHelpers.matchObject({ include: /^foo/ }, "foo.js"); // true
 * ModuleFilenameHelpers.matchObject({ include: [/^foo/, "bar"] }, "foo.js"); // true
 * ModuleFilenameHelpers.matchObject({ include: [/^foo/, "bar"] }, "baz.js"); // false
 * ModuleFilenameHelpers.matchObject({ exclude: "foo.js" }, "foo.js"); // false
 * ModuleFilenameHelpers.matchObject({ exclude: [/^foo/, "bar"] }, "foo.js"); // false
 * ```
 */
export const matchObject = (obj: MatchObject, str: string): boolean => {
	if (obj.test) {
		if (!matchPart(str, obj.test)) {
			return false;
		}
	}
	if (obj.include) {
		if (!matchPart(str, obj.include)) {
			return false;
		}
	}
	if (obj.exclude) {
		if (matchPart(str, obj.exclude)) {
			return false;
		}
	}
	return true;
};
