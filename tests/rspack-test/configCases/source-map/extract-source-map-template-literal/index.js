import { angularStyle, multiline, nested, quoted, escaped, regex } from "./strings";
import { mapped, sameDirective } from "./mapped";
const fs = require("fs");

it("should leave source map directives inside strings unchanged", () => {
	expect(angularStyle("styles.map")).toBe("body {}\n/*# sourceMappingURL=styles.map */");
	expect(multiline).toBe("\n//# sourceMappingURL=missing.map\n");
	expect(nested).toBe("//# sourceMappingURL=missing.map");
	expect(quoted).toBe("/*# sourceMappingURL=missing.map */");
	expect(escaped).toBe("`\n//# sourceMappingURL=missing.map\n");
	expect(regex.test("#")).toBe(true);
});

it("should still extract the real comment after a template literal", () => {
	expect(mapped).toBe("//# sourceMappingURL=missing.map");
	expect(sameDirective).toBe("//# sourceMappingURL=mapped.js.map\n");
	const sourceMap = JSON.parse(fs.readFileSync(__filename + ".map", "utf8"));
	expect(sourceMap.sources.some(source => source.endsWith("/original.js"))).toBe(true);
});
