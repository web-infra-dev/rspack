import { parentPort } from "worker_threads";
import dependency from "./dependency.cjs";

parentPort.postMessage({ thisIsUndefined: this === undefined, dependency });
