import * as barrel from "./barrel";

it("should infer empty exports before an external mutation is added", () => {
	expect(Object.keys(barrel)).not.toContain("value");
});
