const fs = require("fs");
const path = require("path");

const read = file => JSON.parse(fs.readFileSync(path.join(__dirname, file), "utf-8"));

for (const file of ["mf-stats.json", "mf-manifest.json"]) {
	it(`should list every expose key of a shared file in ${file}`, () => {
		const paths = read(file).exposes.map(item => item.path).sort();
		expect(paths).toEqual(["./a", "./a-alias"]);
	});
}

it("should give the aliased expose the same assets", () => {
	const exposes = read("mf-manifest.json").exposes;
	const a = exposes.find(item => item.path === "./a");
	const alias = exposes.find(item => item.path === "./a-alias");
	expect(alias.assets).toEqual(a.assets);
});
