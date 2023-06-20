type CallFn<D> = (args: D[]) => void;

export default class MergeCaller<D> {
	private timer: any = null;
	private callArgs: D[] = [];

	// add in constructor
	private debounceTime: number;
	private callFn: CallFn<D>;
	constructor(fn: CallFn<D>, debounceTime: number) {
		this.debounceTime = debounceTime;
		this.callFn = fn;
	}

	private finalCall = () => {
		this.timer = null;
		const args = this.callArgs;
		this.callArgs = [];
		this.callFn(args);
	};

	push(...data: D[]) {
		if (this.timer) {
			clearTimeout(this.timer);
		}

		this.callArgs.push(...data);

		this.timer = setTimeout(this.finalCall, this.debounceTime);
	}
}
