import * as values from "./shared";

it("should add runtime a to the dynamic import chunk", async () => {
	expect((await values.feature()).value).toBe("feature");
});
