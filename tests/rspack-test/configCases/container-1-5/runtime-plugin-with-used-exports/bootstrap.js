import { Worker } from "worker_threads";
import React from "react";

const msg = 'Hello, Rspack!';

const worker = new Worker(new URL('./worker.js', import.meta.url), {
	name: 'the-worker',
});

export function getWorkerMessage() {
	return new Promise((resolve, reject) => {
		worker.on('message', (data) => {
			resolve(data);
		});

		worker.on('error', (data) => {
			reject(data);
		});

		worker.postMessage(msg);
	});
}

export function getMessage() {
	return `App rendered with [${React()}]`;
}
