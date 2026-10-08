const scriptUrl = new URL(import.meta.resolve("./node_modules/url-entry-target/index.js"), "https://test.cases/path/");
expect(globalThis.URL_ENTRY_SPLIT_TARGET_EXECUTED).toBeUndefined();
const expected = {
  value: "target and dependency executed",
  lazy: "lazy chunk executed",
  asset: true
};

if (URL_ENTRY_LOADING === "jsonp") {
  it("should execute a split URL entry in a document", async () => {
    const executed = new Promise(resolve => {
      globalThis.URL_ENTRY_SPLIT_READY = resolve;
    });
    const script = document.createElement("script");
    script.src = scriptUrl.href;
    let timeout;
    try {
      await new Promise((resolve, reject) => {
        script.onload = resolve;
        script.onerror = reject;
        document.head.appendChild(script);
      });
      const result = await Promise.race([
        executed,
        new Promise((resolve, reject) => {
          timeout = setTimeout(() => reject(new Error("The URL entry loaded but did not execute its target module")), 5000);
        })
      ]);
      expect(result).toEqual(expected);
    } finally {
      clearTimeout(timeout);
      delete globalThis.URL_ENTRY_SPLIT_READY;
      script.remove();
    }
  });
}

it("should execute a split URL entry and its lazy chunk in a worker", async () => {
  // Passing the URL separately exercises URL entries rather than WorkerDependency.
  const workerUrl = new URL(scriptUrl.href, "https://test.cases/path/");
  const worker = new Worker(workerUrl, {
    type: URL_ENTRY_LOADING === "import" ? "module" : "classic"
  });
  try {
    const result = await new Promise((resolve, reject) => {
      worker.onmessage = event => resolve(event.data);
      worker.worker.on("error", reject);
    });
    expect(result).toEqual(expected);
  } finally {
    await worker.terminate();
  }
});
