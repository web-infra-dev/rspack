import wdm from "webpack-dev-middleware";

const rdm: typeof wdm = (compiler, options) => {
	return wdm(compiler, {
		writeToDisk: false,
		...options
	});
};

export default rdm;
