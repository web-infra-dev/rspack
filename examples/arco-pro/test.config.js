module.exports = {
  mode: "development",
  entry: {
    index: ["./src/empty.tsx"],
  },
  loader: {
    json: "json",
    svg: "dataURI",
    less: "text",
    css: "css",
  },
  resolve: {
    alias: {
      "@/": "<ROOT>/src/",
    },
  },
  output: {
    sourceMap: false,
  },
  enhanced: {
    react: {
      fastRefresh: true,
    },
    svgr: true,
  },
};
