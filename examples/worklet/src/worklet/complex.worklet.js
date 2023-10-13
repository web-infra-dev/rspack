import { answer } from "./answer";
class PortProcessor extends AudioWorkletProcessor {
	constructor() {
		super();
		this.port.onmessage = event => {
			// Handling data from the node.
			console.log(event.data);
		};

		this.port.postMessage("Hi!" + answer);
	}

	process(inputs, outputs, parameters) {
		// Do nothing, producing silent output.
		return true;
	}
}

registerProcessor("complex-processor", PortProcessor);
