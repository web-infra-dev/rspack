import { value } from "./a";
export const result = value;

it("updates a star re-export after a direct importer becomes transitively affected", () => {
	expect(result).toBe(Number(WATCH_STEP) === 0 ? "leaf" : "fallback");
});
