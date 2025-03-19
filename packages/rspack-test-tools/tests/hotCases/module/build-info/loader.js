module.exports = async function (content) {
    if (this._module.buildInfo.affected) {
        return "export default new Error();";
    }
    this._module.buildInfo.affected = true;
    return content;
};
