module.exports = function (content) {
    this.emitWarning(new Error("emitted warning1"));
    this.emitError(new Error("emitted error1"));
    this.emitWarning(new Error("emitted warning2"));
    this.emitError(new Error("emitted error2"));

    return content;
}
