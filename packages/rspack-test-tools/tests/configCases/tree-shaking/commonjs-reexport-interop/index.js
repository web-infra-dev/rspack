import { ThemeProvider } from "./lib.js";
it("should compile with static block", function () {
	expect(ThemeProvider()).toBe(20);
});
