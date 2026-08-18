import { JSON as localJSON } from "./local";
import data from "./data.json";

it("should preserve globals referenced by generated JSON modules", () => {
	expect(localJSON()).toBe("local");
	expect(data.message).toBe("generated JSON must use the global parser");
});
