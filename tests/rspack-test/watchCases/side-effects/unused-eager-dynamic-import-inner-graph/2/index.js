import { live, loadEagerFeature } from "./module";
import { leaf } from "./leaf";

it("should restore the eager dynamic import when its export becomes used", async () => {
	expect(live).toBe("live");
	expect(leaf).toBe("after");
	expect((await loadEagerFeature()).value).toBe("WATCH_EAGER_FEATURE_MARKER");
});
