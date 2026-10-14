// exports in the import role, the worker's global scope in the worker role
this.marker = require("./marker");
this.onmessage = event => {
	this.postMessage(`${event.data}:${this.marker}`);
};
