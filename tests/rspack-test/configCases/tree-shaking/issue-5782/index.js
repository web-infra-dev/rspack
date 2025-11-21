import { bbb } from "./package";

it("should override sideEffects in package.json", () => {
	expect(bbb({})).toEqual(undefined);
});
