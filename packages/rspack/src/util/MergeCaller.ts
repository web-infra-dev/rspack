type CallFn<A, C> = (args: Array<[A, C]>) => void;

export default class MergeCaller<A, C> {
	private timer: any = null;
	private callArgs: Array<[A, C]> = [];

	// add in constructor
	private debounceTime: number;
	private callFn: CallFn<A, C>;
	constructor(fn: CallFn<A, C>, debounceTime: number) {
		this.debounceTime = debounceTime;
		this.callFn = fn;
	}

	private finalCall = () => {
		this.timer = null;
		const args = this.callArgs;
		this.callArgs = [];
		this.callFn(args);
	};

	run(a: A, c: C) {
		if (this.timer) {
			clearTimeout(this.timer);
		}

		this.callArgs.push([a, c]);

		this.timer = setTimeout(this.finalCall, this.debounceTime);
	}
}
