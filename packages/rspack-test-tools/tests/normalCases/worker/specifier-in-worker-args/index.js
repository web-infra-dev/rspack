import { Worker } from "worker_threads";
import * as config from "./config";

it("should compile", async () => {
	const worker = new Worker(new URL("./worker", import.meta.url), {
		workerData: config.workerData,
	});
	worker.postMessage("ok");
	const result = await new Promise(resolve => {
		worker.on("message", data => {
			resolve(data);
		});
	});
	expect(result).toBe(config.workerData);
	await worker.terminate();
});
