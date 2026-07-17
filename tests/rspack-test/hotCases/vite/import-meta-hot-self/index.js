import "./self";

it("runs the Vite self callback and preserves hot data", async () => {
	expect(globalThis.__importMetaHotInitial).toEqual({ value: 1, count: 0 });
	await NEXT_HMR();
	expect(globalThis.__importMetaHotAccepted).toEqual({ value: 2, count: 1 });
});
