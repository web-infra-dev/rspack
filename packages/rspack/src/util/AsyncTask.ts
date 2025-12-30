type TaskCallback<Ret> = (err: Error | null, ret: Ret | null) => void;

export class AsyncTask<Param, Ret> {
  #isRunning = false;
  #params: Param[] = [];
  #callbacks: TaskCallback<Ret>[] = [];

  #task: (
    param: Param[],
    callback: (results: [Error | null, Ret | null][]) => void,
  ) => void;

  constructor(
    task: (
      param: Param[],
      callback: (results: [Error | null, Ret | null][]) => void,
    ) => void,
  ) {
    this.#task = task;
  }

  #exec_internal() {
    const params = this.#params;
    const callbacks = this.#callbacks;
    this.#params = [];
    this.#callbacks = [];

    this.#task(params, (results) => {
      this.#isRunning = false;
      if (this.#params.length) {
        this.#isRunning = true;
        queueMicrotask(() => this.#exec_internal());
      }

      for (let i = 0; i < results.length; i++) {
        const [err, result] = results[i];
        const callback = callbacks[i];
        callback(err, result);
      }
    });
  }

  exec(param: Param, callback: TaskCallback<Ret>) {
    if (!this.#isRunning) {
      queueMicrotask(() => this.#exec_internal());
      this.#isRunning = true;
    }

    this.#params.push(param);
    this.#callbacks.push(callback);
  }
}
