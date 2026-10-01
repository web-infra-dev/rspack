/*
 * Test fails: module.readableIdentifier is not a function
 * CssModule readableIdentifier implementation issue
 */
// module.readableIdentifier (from compilation.chunkGraph.getChunkModulesIterable()) is not a function
export default () => "TODO: CssModule of experiments.css";
