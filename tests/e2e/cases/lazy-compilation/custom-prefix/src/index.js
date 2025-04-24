const button = document.createElement("button");
button.textContent = "Click me";
document.body.appendChild(button);

button.addEventListener("click", () => {
  // Dynamic import already contains code that adds component to the page
  // Just use import() and make sure the module is loaded
  import("./component.js").then(() => {
    console.log("Component loaded");
  });
});
