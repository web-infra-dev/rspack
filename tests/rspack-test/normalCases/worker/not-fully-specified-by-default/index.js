import { Worker as MyWorker } from "worker_threads";

it("should compile", () => {
	new MyWorker(new URL("./a", import.meta.url));
	expect(true);
});
