module.exports = function (content) {
    this._module.buildInfo.loaded = true;

    this.emitFile("foo.txt", "foo");
    return content;
};
