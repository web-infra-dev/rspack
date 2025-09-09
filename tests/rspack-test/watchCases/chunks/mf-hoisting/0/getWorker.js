export default () => {
	const worker = new Worker(new URL("./worker", import.meta.url));
	worker.testName = `test worker ${WATCH_STEP}`;
	return worker;
};
