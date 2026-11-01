globalThis.completeHmrWithoutDeltaLoadUnvisited = () => import(/* webpackChunkName: "other" */ "./other");

it("completes newly required chunks even when their factories are unchanged", async () => {
	await import(/* webpackChunkName: "page" */ "./page");
	expect(globalThis.completeHmrWithoutDeltaValue).toBe(2);
	await NEXT_HMR();
	expect(globalThis.completeHmrWithoutDeltaValue).toBe(2);
	await NEXT_HMR();
	expect(globalThis.completeHmrWithoutDeltaValue).toBe(9);
	await NEXT_HMR();
	expect(globalThis.completeHmrWithoutDeltaValue).toBe(9);
	await NEXT_HMR();
	expect(globalThis.completeHmrWithoutDeltaValue).toBe(4);
});

module.hot.accept();
