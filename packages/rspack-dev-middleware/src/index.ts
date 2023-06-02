import wdm from "webpack-dev-middleware";

const rdm: typeof wdm = (compiler, options) => {
	return wdm(compiler, {
		...options,
		writeToDisk: false
	});
};

export default rdm;
