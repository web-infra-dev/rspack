module.exports = {
  plugins: [
    {
      apply() {
        throw new Error("test");
      },
    },
  ],
};
