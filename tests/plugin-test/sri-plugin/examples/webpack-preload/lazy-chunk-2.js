module.exports.test = () => {
  const linkTag = Array.from(document.getElementsByTagName("link")).find(
    (el) => el.rel === "preload"
  );
  const scriptTag = Array.from(document.getElementsByTagName("script")).find(
    (el) => linkTag && el.src === linkTag.href
  );
  console.log(
    scriptTag &&
      linkTag &&
      scriptTag.integrity &&
      scriptTag.integrity === linkTag.integrity
      ? "ok"
      : "error"
  );
};
