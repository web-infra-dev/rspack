module.exports = function (source) {
    this._module.buildInfo.timestamp = Date.now();
    return source;
};
