const fs = require("fs");

const { load: loadLazy } = require("./lazy");
const { load: loadLazyOnce } = require("./lazy-once");
const { load: loadEager } = require("./eager");
const { existing, load: loadWeak } = require("./weak");

it("should fall back to the configured dynamic import modes", async () => {
	expect((await loadLazy("a")).default).toBe("lazy-a");
	expect((await loadLazy("b")).default).toBe("lazy-b");
	expect((await loadLazyOnce("a")).default).toBe("lazy-once-a");
	expect((await loadLazyOnce("b")).default).toBe("lazy-once-b");
	expect((await loadEager("a")).default).toBe("eager-a");
	expect((await loadEager("b")).default).toBe("eager-b");
	expect(existing).toBe("weak-a");
	expect((await loadWeak("a")).default).toBe("weak-a");
	await expect(loadWeak("b")).rejects.toThrow();

	const files = fs.readdirSync(__dirname);
	expect(files.filter(file => /^per-module\d+\.js$/.test(file))).toHaveLength(2);
	expect(files.filter(file => file === "all-modules.js")).toHaveLength(1);
	expect(files.some(file => file.startsWith("unused-eager"))).toBe(false);
	expect(files.some(file => file.startsWith("unused-weak"))).toBe(false);
});
