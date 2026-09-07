import "./connectionless";
import { load, loadSecond, value } from "./route";

it("should reuse stable async outgoings for a transitively affected module", async () => {
	expect(value).toBe(WATCH_STEP === "0" ? "before:stable" : "after:stable");
	expect(globalThis.connectionlessValue).toBe(
		WATCH_STEP === "0" ? "before" : "after"
	);
	expect((await load()).lazy).toBe(true);
	expect((await loadSecond()).lazySecond).toBe(true);
});
