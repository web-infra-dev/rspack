import { parentPort, workerData } from "worker_threads";

parentPort.on("message", ()=> {
	parentPort.postMessage(workerData);
});
