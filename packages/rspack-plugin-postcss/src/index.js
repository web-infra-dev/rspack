const { Processor } = require("postcss");
const pxtorem = require("postcss-pxtorem");

module.exports = async function loader(loaderContext) {
  // TODO: customize options, until js binding support this functionality
  try {
    let root = new Processor([pxtorem]);
    let res = await root.process(loaderContext.source.getCode());
    return {
      content: res.css,
    };
  } catch (err) {
    throw new Error(err);
  }
};
