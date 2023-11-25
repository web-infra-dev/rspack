import simpleWorklet from "./worklet/simple.worklet?url";
import complexWorklet from "./worklet/complex.worklet";
let context = new AudioContext();

context.audioWorklet.addModule(simpleWorklet).then(() => {
	let node = new AudioWorkletNode(context, "port-processor");
	node.port.onmessage = event => {
		// Handling data from the processor.
		console.log(event.data);
	};

	node.port.postMessage("Hello!");
});

context.audioWorklet.addModule(complexWorklet).then(() => {
	let node = new AudioWorkletNode(context, "complex-processor");
	node.port.onmessage = event => {
		// Handling data from the processor.
		console.log(event.data);
	};

	node.port.postMessage("Hello!");
});
