import startsWith from "lodash-es/startsWith.js";

it("should bundle lodash-es through swc-loader", () => {
	expect(startsWith("rspack", "rsp")).toBe(true);
});
