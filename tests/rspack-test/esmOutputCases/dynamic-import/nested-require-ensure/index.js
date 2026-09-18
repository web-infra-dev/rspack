export const load = () => new Promise((resolve, reject) => {
	require.ensure([], () => {
		require.ensure([], async () => {
			const a = await import(/* webpackChunkName: "shared" */ "./a.js");
			const b = await import(/* webpackChunkName: "shared" */ "./b.js");
			resolve([a, b]);
		}, error => reject(error));
	}, error => reject(error));
});

it("should return each module namespace for imports nested in require.ensure", async () => {
	const [a, b] = await load();
	expect({ ...a }).toEqual({ default: "a", name: "module-a", id: 1 });
	expect({ ...b }).toEqual({ default: "b", name: "module-b", id: 2 });
});
