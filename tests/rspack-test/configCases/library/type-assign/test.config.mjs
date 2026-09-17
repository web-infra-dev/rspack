export default {
	afterExecute() {
		delete global.MyLibrary;
	}
};
