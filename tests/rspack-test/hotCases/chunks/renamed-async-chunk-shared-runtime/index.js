import { load } from "./loader";

it("should keep updating a renamed async chunk whose split chunk spans runtimes", async () => {
	await load();
	const value = () => __webpack_require__.c[require.resolveWeak("./page")].exports.value;
	expect(value()).toBe(1);
	// `page` drops `hooks` (renaming its chunk) while `other-page` adds a new
	// module to the `shared` chunk used by both runtimes
	await NEXT_HMR();
	expect(value()).toBe(2);
	await NEXT_HMR();
	expect(value()).toBe(3);
});
