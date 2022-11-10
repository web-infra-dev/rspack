const { transform } = require('@svgr/core');
module.exports = async function svgLoader(loaderContext){
  const svgCode = loaderContext.source.getCode();
  const filePath = loaderContext.resourcePath;
  const componentCode = await transform(svgCode, {}, {
    filePath,
    caller: {
      previousExport: null
    }
  })
  return {
    content: componentCode,
    meta: "",
    map: undefined
  }
}