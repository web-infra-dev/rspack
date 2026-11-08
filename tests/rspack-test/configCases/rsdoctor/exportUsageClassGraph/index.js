import { EntryA } from "./entryA";

const instance = new EntryA();

it("should instantiate EntryA with baz value", () => {
	expect(instance.value).toBe(42);
});
