const { createFsFromVolume, Volume } = require("memfs");
const { TestFs } = require("..");

describe("@rspack/fs", () => {
	const memfs = createFsFromVolume(new Volume());
	const fs = new TestFs({
		writeFile: memfs.writeFileSync.bind(memfs),
		mkdir: memfs.mkdirSync.bind(memfs),
		mkdirp: memfs.mkdirpSync.bind(memfs)
	});

	it("should create a single directory with mkdir", done => {
		fs.mkdirSync("/foo");
		expect(memfs.existsSync("/foo")).toBeTruthy();
		done();
	});

	it("should create directories recursively with mkdirp", done => {
		fs.mkdirpSync("/foo/bar/baz");
		expect(memfs.existsSync("/foo/bar/baz")).toBeTruthy();
		done();
	});

	it("should create a file with given data", done => {
		fs.writeSync("/foo.txt", Buffer.from("Hello World!"));
		expect(memfs.readFileSync("/foo.txt").toString()).toEqual("Hello World!");
		done();
	});
});
