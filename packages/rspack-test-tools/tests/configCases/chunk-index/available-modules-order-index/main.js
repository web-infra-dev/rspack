const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

it("should compile", async () => {
	await (
		await import(/* webpackChunkName: 'A' */ "./A")
	).default;
	await (
		await import(/* webpackChunkName: 'B' */ "./B")
	).default;

	expect(
		fs.readFileSync(path.resolve(__dirname, "./shared.css")).toString()
	).toMatchFileSnapshot(path.join(__SNAPSHOT__, 'shared.css.txt'));
});
