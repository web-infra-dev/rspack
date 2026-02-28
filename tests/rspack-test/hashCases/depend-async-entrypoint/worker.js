import { parentPort } from "worker_threads";

parentPort.on("message", async data => {
	parentPort.postMessage(`pong`);
});
