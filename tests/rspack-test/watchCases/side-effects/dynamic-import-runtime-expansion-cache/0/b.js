import * as values from "./shared";

it("should use the dynamic import in runtime b", async () => {
	expect((await values.feature()).value).toBe("feature");
});
