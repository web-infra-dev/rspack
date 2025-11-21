import * as fs from "node:fs";
import { Worker } from "worker_threads";

it("worker webpackChunkName comments should works", async () => {
	new Worker(new URL(/* webpackChunkName: "chunk-url" */ "./a", import.meta.url));
	new Worker(/* webpackChunkName: "chunk-worker" */ new URL("./a", import.meta.url));
	new Worker(new URL("./a", import.meta.url), { name: "chunk-arg2" });
	const files = await fs.promises.readdir(__dirname);
	expect(files).toContain("chunk-url.bundle0.js");
	expect(files).toContain("chunk-worker.bundle0.js");
	expect(files).toContain("chunk-arg2.bundle0.js");
});
