import { parentPort } from "worker_threads";
import { Y } from "./module";

parentPort.postMessage(Y());
