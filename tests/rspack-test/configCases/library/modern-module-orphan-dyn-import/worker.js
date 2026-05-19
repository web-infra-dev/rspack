self.onmessage = function(e) {
	self.postMessage("worker received: " + e.data);
};
