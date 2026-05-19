import { value } from "./lib/utils";

it("should detect side effects locations via rsdoctor", () => {
	expect(value).toBe(42);
});
