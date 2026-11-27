import { namespace } from "./barrel.js";

it("should keep imports a barrel needs when an `import defer` reexport is requested", () => {
	expect(namespace.value).toBe(1);
});
