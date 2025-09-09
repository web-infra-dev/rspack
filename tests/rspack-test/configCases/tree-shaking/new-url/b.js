import { Worker } from "worker_threads";

export function b() {
	new Worker(new URL("./b.worker.js", import.meta.url));
}
