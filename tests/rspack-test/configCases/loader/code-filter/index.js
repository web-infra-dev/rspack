import direct from "./direct";
import emptyContent from "./empty-content";
import emptyRegexp from "./empty-regexp";
import excludeOnly from "./exclude-only";
import excluded from "./excluded";
import functional from "./functional";
import include from "./include";
import invalidUtf8 from "./invalid-utf8";
import metadata from "./metadata";
import mixedNative from "./mixed-native.txt";
import nativeHit from "./native-hit.txt";
import nativeMiss from "./native-miss";
import pitchShortCircuit from "./pitch-short-circuit";
import regexpA from "./regexp-a";
import regexpB from "./regexp-b";
import rightOutput from "./right-output";
import unicode from "./unicode";

it("should filter normal loader execution using its current input", () => {
	expect(direct).toBe("direct");
	expect(regexpA).toBe("regexp");
	expect(regexpB).toBe("regexp");
	expect(include).toBe("include");
	expect(emptyContent).toBe("empty-content");
	expect(excluded).toBe("__VALUE__");
	expect(functional).toBe("functional");
	expect(excludeOnly).toBe("exclude-only");
	expect(emptyRegexp).toBe("empty-regexp");
	expect(unicode).toBe("unicode");
	expect(rightOutput).toBe("right-output");
	expect(mixedNative).toBe("mixed-native");
});

it("should preserve loader state on a miss or undecodable input", () => {
	expect(metadata).toBe("__VALUE__");
	expect(invalidUtf8).toBe("invalid-preserved");
});

it("should support native loaders and keep pitch behavior", () => {
	expect(nativeHit).toBe("native marker\n-simple");
	expect(nativeMiss).toBe("native");
	expect(pitchShortCircuit).toBe("pitch-short-circuit");
});
