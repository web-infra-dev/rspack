type CallFn<D> = (args: D[]) => void;

export default class MergeCaller<D> {
  private callArgs: D[] = [];

  private callFn: CallFn<D>;
  constructor(fn: CallFn<D>) {
    this.callFn = fn;
  }

  private finalCall = () => {
    const args = this.callArgs;
    this.callArgs = [];
    this.callFn(args);
  };

  pendingData() {
    return this.callArgs;
  }

  push(...data: D[]) {
    if (this.callArgs.length === 0) {
      queueMicrotask(this.finalCall);
    }
    this.callArgs.push(...data);
  }
}
