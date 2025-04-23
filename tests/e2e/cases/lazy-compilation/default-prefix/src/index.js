const button = document.createElement("button");
button.textContent = "Click me";
document.body.appendChild(button);

button.addEventListener("click", () => {
  import("./component.js").then(() => {
    console.log("Component loaded");
  });
});
