import { a, aUsed, aCanBeMangled, aProvided, aToStringProvided, obj, objUsed, objAProvided } from "./reexport";

if (a()) console.log("a", obj);

it("should allow mangle across an empty javascript/auto star reexport", () => {
	expect(aUsed).toBe(true);
	expect(aProvided).toBe(true);
	expect(aCanBeMangled).toBe(true);
	expect(objUsed).toBe(true);
	expect(objAProvided).toBe(undefined);
	expect(aToStringProvided).toBe(undefined);
});
