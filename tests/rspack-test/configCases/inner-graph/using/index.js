import { log } from "./disposable";

it("should keep top-level using declarations whose bindings are unused", async () => {
	const { value } = await import("./module");
	expect(value).toBe(1);
	expect(log).toEqual(["sync", "resource"]);
});
