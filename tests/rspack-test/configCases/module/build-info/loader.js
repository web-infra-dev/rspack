module.exports = function (content) {
    this._module.buildInfo.loaded = true;

    this.emitFile("foo.txt", "foo");
    this.addBuildDependency("./build.txt");
    return content;
};
