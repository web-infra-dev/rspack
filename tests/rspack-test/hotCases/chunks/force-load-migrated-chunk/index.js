import { getV } from "./mod";

module.hot.accept("./mod");

it("should force-load a migrated chunk so the module keeps working", async () => {
	expect(getV()).toBe(1);
	// `mod` now imports `shared` dynamically, so `shared` leaves main's shared
	// chunk for the "lazy" async chunk, which must be force-loaded
	await NEXT_HMR();
	expect(await getV()).toBe(1);
});
