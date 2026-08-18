import { a as escaped } from "./escaped";
import {
	a,
	原始拼接全局引用 as localUnicode,
	转义拼接全局引用 as localEscapedUnicode
} from "./plain";

it("should preserve canonical names for escaped identifiers", () => {
	expect(escaped).toBe("escaped");
	expect(a).toBe("plain");
});

it("should reserve Unicode names referenced by generated code", () => {
	expect(GENERATED_UNICODE).toBe("undefined");
	expect(GENERATED_ESCAPED_UNICODE).toBe("undefined");
	expect(localUnicode).toBe("local raw");
	expect(localEscapedUnicode).toBe("local escaped");
});
