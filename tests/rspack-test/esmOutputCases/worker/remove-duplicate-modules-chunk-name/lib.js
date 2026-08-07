import { parentPort } from "node:worker_threads";

const namedWorkerModuleMarker = "worker result";

if (parentPort) parentPort.postMessage(namedWorkerModuleMarker);

export default "lib";
