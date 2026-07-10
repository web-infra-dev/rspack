import { unused, unprovided } from "./unused";
import {
	value as valueStatic,
	valueUsed as valueUsedStatic
} from "./dedupe-target-static";
import {
	value as valueSide,
	valueUsed as valueUsedSide
} from "./dedupe-target-with-side";
import { value, valueUsed } from "./dedupe-target";
import * as DefaultExport from "./default-export";
import {
	value as valueDirect,
	value2 as value2Direct,
	default as Default1
} from "./direct-export";
import {
	value as valueChecked,
	value2 as value2Checked
} from "./checked-export";
import Default2 from "./dynamic-reexport-default";
import {
	value as valueMultipleSources,
	value2 as value2MultipleSources
} from "./multiple-sources";
import * as SharedDynamicRuntime from "./shared-dynamic-runtime";
import { a, b } from "./swapped";

it("should dedupe static reexport target", () => {
	expect(valueStatic).toBe(42);
	expect(valueUsedStatic).toBe(unused);
});

it("should dedupe dynamic reexport target", () => {
	expect(value).toBe(undefined);
	expect(valueUsed).toBe(unused);
});

it("should not dedupe dynamic reexport target when it has side-effects", () => {
	expect(valueSide).toBe(undefined);
	expect(valueUsedSide).toBe(true);
});

it("should optimize dynamic default reexport", () => {
	expect(DefaultExport.a).toBe(42);
	expect(DefaultExport.b).toBe(42);
	expect(DefaultExport.empty).toEqual({});
	expect(DefaultExport.json).toBe(42);
});

it("should handle default export when reexporting", () => {
	const module = Object(require("./reexports-excludes-default"));
	expect(module.defaultProvided).toBe(unprovided);
});

it("should handle direct export when reexporting", () => {
	expect(valueDirect).toBe(42);
	expect(value2Direct).toBe(42);
});

it("should handle checked dynamic export when reexporting", () => {
	expect(valueChecked).toBe(42);
	expect(value2Checked).toBe(42);
});

it("should handle default export correctly", () => {
	expect(Default1).toBe(undefined);
	expect(Default2).toBe("static");
});

it("should handle multiple dynamic sources correctly", () => {
	expect(valueMultipleSources).toBe(42);
	expect(value2MultipleSources).toBe(42);
	expect(
		Object.fromEntries(
			Object.keys(SharedDynamicRuntime).map(key => [
				key,
				SharedDynamicRuntime[key]
			])
		)
	).toEqual({
		a: 1,
		b: 2,
		c: 3,
		d: 4,
		e: 5,
		f: 6,
		g: 7,
		h: 8,
		i: 9,
		j: 10,
		k: 11,
		l: 12,
		m: 13,
		n: 14,
		o: 15,
		p: 16
	});
});

it("should handle renamed dynamic reexports", () => {
	expect(a).toBe(43);
	expect(b).toBe(42);
});

it("should use shared runtime for repeated dynamic reexports", () => {
	const source = require("fs").readFileSync(
		require("path").join(__STATS__.outputPath, "bundle.js"),
		"utf-8"
	);
	expect(source).toContain("__rspack_" + "reexport");
	expect(source.split("__webpack_require__" + ".re(")).toHaveLength(17);
});
