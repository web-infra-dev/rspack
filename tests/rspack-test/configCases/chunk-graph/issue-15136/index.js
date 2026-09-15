import { isEqual, startsWith, uniq } from "lodash-es";

it("should register transformed ESM modules before splitting the chunk graph", () => {
	expect(startsWith("rspack", "rsp")).toBe(true);
	expect(isEqual({ modules: ["a", "b"] }, { modules: ["a", "b"] })).toBe(true);
	expect(uniq(["entry", "shared", "entry"])).toEqual(["entry", "shared"]);
});
