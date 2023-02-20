const { createFsFromVolume, Volume } = require("memfs");
const { promisify } = require("util");
const fs = require("..");

describe("@rspack/fs", () => {
	const memfs = createFsFromVolume(new Volume());

	it("should create a single directory with mkdir", done => {
		fs.writeFile();
		done();
	});
});

const memfs = createFsFromVolume(new Volume());
fs.writeFile(memfs.writeFileSync.bind(memfs));
const res = memfs.readFileSync("/foo.txt");
console.log(res.toString());
