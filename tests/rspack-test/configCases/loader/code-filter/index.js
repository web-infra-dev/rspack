import array from "./array";
import emptyContent from "./empty-content";
import emptyExclude from "./empty-exclude";
import emptyInclude from "./empty-include";
import emptyString from "./empty-string";
import excluded from "./excluded";
import functional from "./functional";
import invalidUtf8 from "./invalid-utf8";
import metadata from "./metadata";
import mixedNative from "./mixed-native.txt";
import nativeHit from "./native-hit.txt";
import nativeMiss from "./native-miss";
import pitchShortCircuit from "./pitch-short-circuit";
import regexpA from "./regexp-a";
import regexpB from "./regexp-b";
import rightOutput from "./right-output";
import string from "./string";
import unicode from "./unicode";

it("should filter normal loader execution using its current input", () => {
	expect(string).toBe("string");
	expect(regexpA).toBe("regexp");
	expect(regexpB).toBe("regexp");
	expect(array).toBe("array");
	expect(emptyContent).toBe("empty-content");
	expect(excluded).toBe("__VALUE__");
	expect(functional).toBe("functional");
	expect(emptyInclude).toBe("__VALUE__");
	expect(emptyExclude).toBe("empty-exclude");
	expect(emptyString).toBe("empty-string");
	expect(unicode).toBe("unicode");
	expect(rightOutput).toBe("right-output");
	expect(mixedNative).toBe("mixed-native");
});

it("should preserve loader state on a miss", () => {
	expect(metadata).toBe("__VALUE__");
	expect(invalidUtf8).toBe("invalid-preserved");
});

it("should support native loaders and keep pitch behavior", () => {
	expect(nativeHit).toBe("native marker\n-simple");
	expect(nativeMiss).toBe("native");
	expect(pitchShortCircuit).toBe("pitch-short-circuit");
});
