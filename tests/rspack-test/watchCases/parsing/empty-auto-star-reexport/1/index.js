import { dynamic } from "./barrel";

it("restores dynamic star reexports after the target changes", () => {
	expect(dynamic).toBe("dynamic");
});
