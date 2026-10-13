import { angularStyle, multiline, nested, quoted, escaped, regex } from "./strings";
import { mapped, sameDirective } from "./mapped";
import { guarded } from "./guarded";
import { nonTrailing } from "./non-trailing";
import { value as single } from "./guard-single";
import { value as backtick } from "./guard-backtick";
import { value as terminator } from "./guard-terminator";
import { value as directive } from "./guard-directive";
import { value as inline } from "./guard-inline";
import { value as lf } from "./line-lf";
import { value as crlf } from "./line-crlf";
import { value as cr } from "./line-cr";
import { value as ls } from "./line-ls";
import { value as ps } from "./line-ps";
import { block } from "./block";
import regexBoundary from "./regex-boundary";
import { regexTail } from "./regex-tail";
import { bom } from "./bom";
import { bomLeading } from "./bom-leading";
import nestedBlock from "./nested-block";
import templateBlockTail from "./template-block-tail";
import "./mapped.css";
import "./nested.css";
const fs = require("fs");

it("should not resolve directives before ambiguous tail comments or code", () => {
	expect(guarded).toBe("guarded");
	expect(nonTrailing).toBe("non-trailing");
	expect(single).toBe("single");
	expect(backtick).toBe("backtick");
	expect(terminator).toBe("terminator");
	expect(directive).toBe("directive");
	expect(inline).toBe("inline");
	expect(regexBoundary.source).toBe(".*");
	expect(regexTail.source).toBe(".*");
	expect(nestedBlock).toBe(1);
	expect(templateBlockTail).toBe("\n//# sourceMappingURL=mapped.js.map\n/* ordinary\n");
});

it("should accept ECMAScript BOM whitespace around trailing directives", () => {
	expect(bom()).toBe("bom");
	expect(bomLeading()).toBe("bom-leading");
	const sourceMap = JSON.parse(fs.readFileSync(__filename + ".map", "utf8"));
	expect(sourceMap.sources.some(source => source.endsWith("/bom-original.js"))).toBe(true);
	expect(sourceMap.sources.some(source => source.endsWith("/bom-leading-original.js"))).toBe(true);
});

it("should extract real trailing directives across JavaScript line terminators", () => {
	expect([lf(), crlf(), cr(), ls(), ps()]).toEqual(["lf", "crlf", "cr", "ls", "ps"]);
	const sourceMap = JSON.parse(fs.readFileSync(__filename + ".map", "utf8"));
	for (const original of ["line-lf-original.js", "line-crlf-original.js", "line-cr-original.js", "line-ls-original.js", "line-ps-original.js"]) {
		expect(sourceMap.sources.some(source => source.endsWith("/" + original))).toBe(true);
	}
});

it("should retain trailing JavaScript and CSS block directives", () => {
	expect(block()).toBe("block");
	const jsSourceMap = JSON.parse(fs.readFileSync(__filename + ".map", "utf8"));
	expect(jsSourceMap.sources.some(source => source.endsWith("/block-original.js"))).toBe(true);
	const sourceMap = JSON.parse(fs.readFileSync(__filename.replace(/\.js$/, ".css.map"), "utf8"));
	expect(sourceMap.sources.some(source => source.endsWith("/original.css"))).toBe(true);
	expect(sourceMap.sources.some(source => source.endsWith("/nested.css"))).toBe(true);
});

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
