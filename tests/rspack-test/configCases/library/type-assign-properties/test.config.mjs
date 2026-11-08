export default {
	afterExecute() {
		delete global.MyLibraryProperties;
	}
};
