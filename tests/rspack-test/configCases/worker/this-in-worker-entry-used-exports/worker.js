this.api = require("./api");
self.onmessage = () => {
	self.postMessage(self.api.answer);
};
