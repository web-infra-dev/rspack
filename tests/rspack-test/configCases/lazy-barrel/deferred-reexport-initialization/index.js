import { other } from "./barrel.js";

it("should keep imports a barrel needs once an `import defer` reexport is requested", () => {
	expect(other).toBe(3);
	return import("./later.js").then(({ read }) => {
		expect(read()).toBe(1);
	});
});
