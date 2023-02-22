const { createFsFromVolume, Volume } = require("memfs");
const { TestFs, TestThreadsafeFs } = require("..");

function isPromiseLike(value) {
	return (
		typeof value === "object" &&
		value !== null &&
		typeof value.then === "function"
	);
}

const assertPromise = async value => {
	expect(isPromiseLike(value)).toBeTruthy();
	return await value;
};

describe("@rspack/fs", () => {
	let memfs;
	let fs;
	let fsPromise;

	beforeEach(() => {
		memfs = createFsFromVolume(new Volume());
		fs = new TestFs({
			writeFile: memfs.writeFileSync.bind(memfs),
			mkdir: memfs.mkdirSync.bind(memfs),
			mkdirp: memfs.mkdirpSync.bind(memfs)
		});
		fsPromise = new TestThreadsafeFs({
			writeFile: memfs.writeFileSync.bind(memfs),
			mkdir: memfs.mkdirSync.bind(memfs),
			mkdirp: memfs.mkdirpSync.bind(memfs)
		});
	});

	describe("sync", () => {
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

	describe("async", () => {
		it("should create a single directory with mkdir", async () => {
			await assertPromise(fsPromise.mkdir("/foo"));
			expect(memfs.existsSync("/foo")).toBeTruthy();
		});

		it("should create directories recursively with mkdirp", async () => {
			await assertPromise(fsPromise.mkdirp("/foo/bar/baz"));
			expect(memfs.existsSync("/foo/bar/baz")).toBeTruthy();
		});

		it("should create a file with given data", async () => {
			await assertPromise(
				fsPromise.write("/foo.txt", Buffer.from("Hello World!"))
			);
			expect(memfs.readFileSync("/foo.txt").toString()).toEqual("Hello World!");
		});
	});
});
