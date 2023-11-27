import { a } from "./star*/a";

it("should generate valid code", async () => {
	expect(a).toBe(1);
	expect((await import("./star*/a")).a).toBe(1);
});
