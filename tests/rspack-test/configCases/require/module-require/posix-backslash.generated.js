import { createRequire as _createRequire } from "module";

it("should create require from absolute file URL object", () => {
	expect(_createRequire(new URL("file:///data00/home/jinzhixin/rstack/rspack/tests/rspack-test/configCases/require/module-require/foo/c.js"))("./a")).toBe(4);
});

it("should create require from absolute file URL object with ignored base", () => {
	expect(_createRequire(new URL("file:///data00/home/jinzhixin/rstack/rspack/tests/rspack-test/configCases/require/module-require/foo/c.js", undefined))("./a")).toBe(4);
});

it("should normalize direct file URL dot segments", () => {
	expect(_createRequire("file:///data00/home/jinzhixin/rstack/rspack/tests/rspack-test/configCases/require/module-require/foo/..")("./a")).toBe(1);
});

it("should accept normalized file URL object spellings", () => {
	expect(_createRequire(new URL("file:/data00/home/jinzhixin/rstack/rspack/tests/rspack-test/configCases/require/module-require/foo/c.js", import.meta.url))("./a")).toBe(4);
});


it("should treat POSIX absolute paths ending in backslash as files", () => {
	expect(_createRequire(__dirname + "/foo\\")("./posix-backslash")).toBe(
		"posix-backslash"
	);
});
