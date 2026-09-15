module.exports = {
  mode: 'production',
  devServer: {
    port: Number(process.env.RSPACK_PREVIEW_TEST_PORT),
  },
};
