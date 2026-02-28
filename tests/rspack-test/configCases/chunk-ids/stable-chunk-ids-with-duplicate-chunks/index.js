export default async () => {
	const { test } = await import(/* webpackMode: "eager" */'./module')

	test()
};

it("should have stable chunkIds and chunk content", async () => {
	const fs = __non_webpack_require__("fs");
	const path = __non_webpack_require__("path");
	const files = (await fs.promises.readdir(__dirname)).filter(file => file.startsWith("node_modules_cell_index_js-") || file.startsWith("node_modules_row_index_js-"));
	let snapshot = "";
	for (const file of files) {
		const key = file.replace(/(.*)-(.*)(\d\.bundle0\.js)/, "$1-XXX$3");
		const content = await fs.promises.readFile(path.resolve(__dirname, file), "utf-8");
		snapshot += `${key}\n\n::\n\n${content}\n`;
		snapshot += '==============================================================\n';
	}
	expect(snapshot).toMatchFileSnapshotSync(path.join(__SNAPSHOT__, 'snapshot.txt'));
})
