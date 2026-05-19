module.exports = async (url) => {
  const pathname = new URL(url).pathname;

  if (pathname === "/value.js") {
    return {
      status: 200,
      headers: {
        "content-type": "application/javascript",
      },
      body: Buffer.from("export const named = 42; export default 42;\n"),
    };
  }

  return {
    status: 404,
    headers: {
      "content-type": "text/plain",
    },
    body: Buffer.from(`Not found: ${pathname}`),
  };
};
