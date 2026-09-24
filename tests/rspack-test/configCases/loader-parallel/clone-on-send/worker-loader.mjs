export default function () {
  return `module.exports = ${this._module.buildInfo.payload.value};`;
}
