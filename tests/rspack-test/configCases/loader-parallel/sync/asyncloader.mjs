export default function (content) {
	return new Promise(resolve => {
		setTimeout(() => {
			resolve(content);
		});
	});
};
