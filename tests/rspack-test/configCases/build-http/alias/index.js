import runtimeFromUrl from "https://test.rspack.rs/runtime.js";
import runtimeFromAlias from "./local-consumer";

it("should resolve an alias to the same HTTP module", () => {
	expect(runtimeFromAlias).toBe(runtimeFromUrl);
});
