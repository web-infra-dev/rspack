type CallFn<D> = (args: D[]) => void;

export default class MergeCaller<D> {
	private microtask: Promise<void> | null = null;
	private callArgs: D[] = [];

	private callFn: CallFn<D>;
	constructor(fn: CallFn<D>) {
		this.callFn = fn;
	}

	private finalCall = () => {
		this.microtask = null;
		const args = this.callArgs;
		this.callArgs = [];
		this.callFn(args);
	};

	push(...data: D[]) {
		if (!this.microtask) {
			this.microtask = Promise.resolve();
			this.microtask.then(this.finalCall);
		}

		this.callArgs.push(...data);
	}
}
