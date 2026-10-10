import { load } from "./loader";

it("should provide re-imported split chunk modules before the renamed chunk's module re-runs", async () => {
	await load();
	const value = () => __webpack_require__.c[require.resolveWeak("./page")].exports.value;
	expect(value()).toBe("v1-1");
	// page drops ./hooks: its chunk is renamed and the hooks chunk removed
	await NEXT_HMR();
	expect(value()).toBe("v2");
	// page imports ./hooks again: hooks and policy must exist when page re-runs
	await NEXT_HMR();
	expect(value()).toBe("v3-1");
	// policy's update reaches the client, so the restored chunks are installed
	await NEXT_HMR();
	expect(value()).toBe("v3-2");
});
