import * as ns from "./reexport";
import * as alias from "./alias";
import * as root from "./root";
import * as defaultNs from "./default-reexport";
import * as chainNs from "./chain";
import * as cjsRequire from "cjs-require-lib";
import { strictResults } from "./strict.mjs";

it("should evaluate `in` on a skipped namespace re-export to true", () => {
	expect("inner" in ns).toBe(true);
	expect("aliased" in alias).toBe(true);
	expect("inner" in root.mid).toBe(true);
	expect("q" in ns.inner).toBe(true);
	expect("missing" in ns.inner).toBe(false);
});

it("should evaluate `in` on a namespace re-export of every module type to true", () => {
	expect("cjsInner" in ns).toBe(true);
	expect("flaggedInner" in ns).toBe(true);
	expect("jsonInner" in ns).toBe(true);
	expect("effectInner" in ns).toBe(true);
	expect("dynamicInner" in ns).toBe(true);
	expect("fnInner" in ns).toBe(true);
});

it("should evaluate `in` on every form of namespace re-export to true", () => {
	expect("default" in defaultNs).toBe(true);
	expect("inner" in chainNs).toBe(true);
	expect("inner" in cjsRequire).toBe(true);
});

it("should evaluate `in` on namespace re-exports in a strict ESM module to true", () => {
	expect(strictResults).toEqual({
		inner: true,
		cjsInner: true,
		flaggedInner: true,
		jsonInner: true,
		effectInner: true,
		dynamicInner: true,
		fnInner: true,
		default: true,
		chained: true,
		cjsRequire: true
	});
});

it("should keep the branch guarded by `in` on a skipped namespace re-export", () => {
	let result = 0;
	if ("inner" in ns) {
		result = ns.inner.q;
	}
	expect(result).toBe(1);
});
