const expect = require("expect");
const { createIntegrityPlugin, getDist } = require("../wsi-test-helper");
module.exports = {
  entry: "./index.js",
  output: {
    filename: "[name].js",
    chunkFilename: "[contenthash].chunk.js",
    crossOriginLoading: "anonymous",
    path: getDist(__dirname),
  },
  devtool: false,
  optimization: {
    moduleIds: "deterministic",
    realContentHash: true,
    chunkIds: "deterministic",
    runtimeChunk: "single",
  },
  plugins: [
    createIntegrityPlugin({
      hashFuncNames: ["sha256"],
      enabled: true,
    }),
    {
      apply: (compiler) => {
        compiler.hooks.done.tap("wsi-test", (stats) => {
          const mainAsset = stats.compilation.assets["main.js"];
          expect(mainAsset.source()).toContain(`JSON.parse('{"key":"value","key2":"value2","key3":"value","key4":"value2","key5":"value","key6":"value2","key7":"value","key8":"value2","key9":"value","key10":"value2","key11":"value","key12":"value2","key13":"value","key14":"value2","key15":"value","key16":"value2","key17":"value","key18":"value2","key19":"value","key20":"value2","key21":"value","key22":"value2"}')`);
        });
      },
    },
  ],
};
