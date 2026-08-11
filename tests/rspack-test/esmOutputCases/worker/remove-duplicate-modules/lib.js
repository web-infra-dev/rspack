import { parentPort } from "node:worker_threads";

const workerModuleMarker = "worker result";

if (parentPort) parentPort.postMessage(workerModuleMarker);

export default "lib";
