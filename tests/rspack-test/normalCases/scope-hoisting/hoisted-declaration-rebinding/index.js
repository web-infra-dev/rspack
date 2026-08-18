import {
	value as removedValue,
	data as removedData,
	a as removedA
} from "./removed";
import { value as directValue } from "./direct";

it("should preserve bindings recreated by dead-code elimination", () => {
	expect(removedValue).toBeUndefined();
	expect(removedData).toBeUndefined();
	expect(removedA).toBeUndefined();
	expect(directValue).toBe("direct");
});
