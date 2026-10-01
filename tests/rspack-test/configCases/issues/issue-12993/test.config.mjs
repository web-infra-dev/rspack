export default {
	afterExecute() {
		delete global.lib;
	}
};
