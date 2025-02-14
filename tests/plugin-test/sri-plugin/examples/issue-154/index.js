import "./style.css";

window.addEventListener("load", () => {
  const { integrity } = document.querySelector("link");
  const loaded = getComputedStyle(document.body).background.match(
    /rgb\(255, 0, 0\)/
  );

  console.log(integrity && loaded ? "ok" : "error");
});
