self.onmessage = ({ data: { question } }) => {
	self.postMessage({
		answer: 42
	});
};
