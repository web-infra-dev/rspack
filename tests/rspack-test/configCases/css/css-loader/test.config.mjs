export default {
  moduleScope(scope, stats) {
    const index = stats().__index__ ?? 0;
    if (index !== 0) return;
    if (scope.window.document.querySelector('[data-css-loader-test]')) return;
    const link = scope.window.document.createElement("link");
    link.setAttribute("data-css-loader-test", "");
    link.rel = "stylesheet";
    link.href = `bundle${index}.css`;
    scope.window.document.head.appendChild(link);
  }
};
