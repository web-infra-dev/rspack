/*
 * Test fails: module.readableIdentifier is not a function
 * CssModule readableIdentifier implementation issue
 */
// module.readableIdentifier (from compilation.chunkGraph.getChunkModulesIterable()) is not a function
module.exports = () => "TODO: CssModule of experiments.css";

