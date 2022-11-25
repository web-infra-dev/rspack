const { transform } = require('@svgr/core');
module.exports = function svgLoader(content){
  const callback = this.async();
  const filePath = this.resourcePath;

  transform(content, {}, {
    filePath,
    caller: {
      previousExport: null
    }
  }).then((componentCode) => {
    callback(null, componentCode);
  })
}