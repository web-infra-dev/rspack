it("should resolve when import existed chunk (#8626)", () => new Promise((resolve, reject) => {
	  const done = err => (err ? reject(err) : resolve());
	require.context("./dir-initial/");
	const fileName = "initialModule";
	import(`./dir-initial/${fileName}`).then(({default:m}) => {
		expect(m).toBe("initialModuleDefault");
		done();
	}).catch(done);
}));

it("should resolve when import existed chunk with fake maps", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	require.context("./dir-initial-with-fake-map/");
	const fileName = "initialModule";
	import(`./dir-initial-with-fake-map/${fileName}`).then(({default:m}) => {
		expect(m).toBe("initialModuleDefault");
		done();
	}).catch(done);
}));
