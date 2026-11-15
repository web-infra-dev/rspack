import { value as direct } from "./direct";
import { value as transitive } from "./barrel";

// Keep both specifier connections active; the minifier removes this function.
function unused() {
	return [direct, transitive];
}

it("should invalidate cached import annotations when dependency side effects change", async () => {
	expect(globalThis.pureImportCacheEffects || []).toEqual(
		COMPILER_INDEX === 2 ? ["direct", "transitive"] : []
	);
	delete globalThis.pureImportCacheEffects;
	if (COMPILER_INDEX < 3) {
		await NEXT_START();
	}
});
