import { installRemoteHarness } from "./remoteHarness";

const harness = installRemoteHarness();

function getFederationInstance() {
	const instance = __webpack_require__.federation?.instance;
	expect(instance).toBeTruthy();
	expect(typeof instance.clearCache).toBe("function");
	return instance;
}

async function clearRemoteA() {
	await getFederationInstance().clearCache({ name: "remoteA" });
}

async function renderPageA() {
	const { render } = await import("./pageA");
	return render();
}

async function renderPageB() {
	const { render } = await import("./pageB");
	return render();
}

async function renderBothPages() {
	return {
		pageA: await renderPageA(),
		pageB: await renderPageB()
	};
}

it("should invalidate SSR remote and affected consumer caches without preloading", async () => {
	expect(await renderBothPages()).toEqual({
		pageA: "pageA:./A:v1",
		pageB: "pageB:./B:v1"
	});
	expect(harness.routeExecutions).toEqual({
		pageA: 1,
		pageB: 1
	});

	harness.setVersion("v2");
	const beforeClear = harness.snapshot();
	await clearRemoteA();

	expect(harness.snapshot()).toEqual(beforeClear);

	expect(await renderBothPages()).toEqual({
		pageA: "pageA:./A:v2",
		pageB: "pageB:./B:v2"
	});
	expect(harness.routeExecutions).toEqual({
		pageA: 2,
		pageB: 2
	});
	expect(harness.entryLoads.length).toBeGreaterThan(beforeClear.entryLoads);
	expect(harness.gets.length).toBeGreaterThan(beforeClear.gets);
});

it("should prevent pending old remote load from updating future caches", async () => {
	harness.setVersion("v3");
	await clearRemoteA();

	harness.blockNextRemoteGet();
	const oldRequest = renderPageA();
	await harness.waitForPendingGet();

	const beforeClear = harness.snapshot();
	harness.setVersion("v4");
	const clearPromise = clearRemoteA();

	expect(harness.snapshot()).toEqual(beforeClear);

	harness.resolvePendingGet();
	await expect(oldRequest).resolves.toBe("pageA:./A:v3");
	await clearPromise;

	expect(await renderPageA()).toBe("pageA:./A:v4");
});
