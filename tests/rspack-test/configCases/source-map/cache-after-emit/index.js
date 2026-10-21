import { value } from "./value";

it("keeps emitted sources readable after releasing their caches", () => {
	expect(value).toBe("中文😀");
});
