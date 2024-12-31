module.exports = function (content) {
    this._module.buildInfo.loaded = true;
    return content;
};
