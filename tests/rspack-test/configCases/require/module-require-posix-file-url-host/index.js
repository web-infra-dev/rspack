import { createRequire as _createRequire } from "module";

it("should not parse non-local createRequire file URL hosts", () => {
	expect(() => _createRequire("file://example.com/project/foo.js")("./a")).toThrow();
	expect(() => _createRequire(new URL("//example.com/project/foo.js", import.meta.url))("./a")).toThrow();
});
