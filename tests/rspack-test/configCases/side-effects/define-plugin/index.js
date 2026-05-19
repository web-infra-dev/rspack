import { f } from "./reexport";

it("should not include reexport modules", async () => {
	const content = await __non_webpack_require__("fs/promises").readFile(__filename, "utf-8");
	expect(f()).toBe(42);
	const sideEffectsFreeModules = ["./lib/index" + ".js", "./reexport" + ".js"];
	for (const module of sideEffectsFreeModules) {
		expect(content).not.toContain(module);
	}
});
