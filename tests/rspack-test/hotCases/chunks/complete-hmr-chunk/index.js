globalThis.completeHmrChunkLoadUnvisited = () => import(/* webpackChunkName: "other" */ "./other");

it("keeps a completely installed chunk subscribed after its importer stops using it", async () => {
	await import(/* webpackChunkName: "page" */ "./page");
	expect(globalThis.completeHmrChunkValue).toBe(1);
	await NEXT_HMR();
	expect(globalThis.completeHmrChunkValue).toBe(2);
	await NEXT_HMR();
	expect(globalThis.completeHmrChunkValue).toBe(9);
	await NEXT_HMR();
	expect(globalThis.completeHmrChunkValue).toBe(9);
	await NEXT_HMR();
	expect(globalThis.completeHmrChunkValue).toBe(4);
	expect(globalThis.completeHmrChunkUnvisitedExecuted).toBeUndefined();
});

module.hot.accept();
