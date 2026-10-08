// This stress case exercises native stack growth, which SWC disables on wasm32.
export default () => !process.env.WASM;
