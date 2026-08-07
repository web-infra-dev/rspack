import { LongExportName } from "./barrel";

it("handles an empty star reexport", () => {
	expect(LongExportName).toBe("value");
});
