export default function () {
  return `module.exports = ${JSON.stringify({
    hookData: this.data.hookData,
    loaderIndex: this.loaderIndex,
  })};`;
};
