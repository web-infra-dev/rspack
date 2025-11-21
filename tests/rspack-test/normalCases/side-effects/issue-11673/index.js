import { Worker } from "worker_threads";
import { X } from "./module";
// test

it("should compile", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	expect(X()).toBe("X");
	const worker = new Worker(new URL("worker.js", import.meta.url));
	worker.once("message", value => {
		expect(value).toBe(42);
		Promise.resolve(worker.terminate()).then(() => done(), done);
	});
}));
