import rspackEsmDefault from "@rspack/core";
import assert from "node:assert";

/**
 * IsAny function is copied from
 * https://github.com/mmkal/expect-type/blob/1e371161eeebcd34c0c7e0806eb9dfa02bc0a509/src/utils.ts#L49
 * @license Apache-2.0
 */
type Not<T extends boolean> = T extends true ? false : true;
const secret = Symbol("secret");
type Secret = typeof secret;
type IsNever<T> = [T] extends [never] ? true : false;
type IsAny<T> = [T] extends [Secret] ? Not<IsNever<T>> : false;

describe("js-api-type should be correct when importing from @rspack/core", () => {
	it("esm default import", async () => {
		type Falsy = IsAny<typeof rspackEsmDefault>;
		const falsy: Falsy = false;
		falsy;
		assert(rspackEsmDefault["BannerPlugin"]);
	});
});
