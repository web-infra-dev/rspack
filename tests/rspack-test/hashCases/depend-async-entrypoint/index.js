import { Worker } from "worker_threads";

const worker = new Worker(/* webpackChunkName: "worker" */new URL("./worker", import.meta.url));
worker;
