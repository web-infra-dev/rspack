import { getCjsFoo } from "./cjs-user";
import { callMethod } from "./callable-user";
import { getEmptyDestructure } from "./empty-destructure-user";
import { getNormalReexportUsed } from "./normal-user";
import { getNamespaceObject } from "./namespace-whole-user";
import { getNamespaceValue } from "./namespace-user";
import { getOverlapStar } from "./overlap-star-user";
import { getStarNestedUsed } from "./star-nested-user";
import { getUndefinedReexport } from "./undefined-user";
import { getAssetUrl } from "./url-user";
import { foo } from "./lib";
import { getJsonName } from "./json-user";

it("should collect rsdoctor export usage graph", () => {
	expect(foo()).toBe(42);
	expect(getJsonName()).toBe("rspack");
	expect(getCjsFoo()).toBe(3);
	expect(callMethod()).toBe("callable");
	expect(getEmptyDestructure()).toBe("empty-destructure");
	expect(getNamespaceValue()).toBe("ns-value");
	expect(getNamespaceObject().nsValue).toBe("ns-value");
	expect(getNormalReexportUsed()).toBe("normal-used");
	expect(getOverlapStar()).toBe("overlap-foo-aoverlap-bar");
	expect(getStarNestedUsed()).toBe("star-nested-used-local");
	expect(getUndefinedReexport()).toBe(undefined);
	expect(getAssetUrl()).toContain("url-asset.wasm");
});
