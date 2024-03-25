const fs = require("fs");
const path = require("path");

it("should compile", async () => {
	await (
		await import(/* webpackChunkName: 'A' */ "./A")
	).default;
	await (
		await import(/* webpackChunkName: 'B' */ "./B")
	).default;

	expect(
		fs.readFileSync(path.resolve(__dirname, "./shared.css")).toString()
	).toMatchSnapshot();
});
