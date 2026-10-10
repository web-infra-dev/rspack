import { load } from "./loader";

it("should keep updating a loaded async module after its chunk is renamed", async () => {
	await load();
	const value = () => __webpack_require__.c[require.resolveWeak("./page")].exports.value;
	expect(value()).toBe(1);
	await NEXT_HMR();
	expect(value()).toBe(2);
	await NEXT_HMR();
	expect(value()).toBe(3);
});
