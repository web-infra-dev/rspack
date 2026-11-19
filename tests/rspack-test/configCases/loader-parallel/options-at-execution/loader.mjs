import { threadId } from "node:worker_threads";

export default function (source) {
	const options = this.getOptions();
	if (options.getCompilerThread) {
		return `exports.inWorker = ${options.getCompilerThread() !== threadId};`;
	}
	this._module.buildInfo.workerOption = options.value;
	return `${source}\nexports.value = ${JSON.stringify(options.value)};`;
}
