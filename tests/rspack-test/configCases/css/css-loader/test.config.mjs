export default {
  moduleScope(scope, stats) {
    const index = stats().__index__ ?? 0;
    if (index !== 0) return;
    const link = scope.window.document.createElement("link");
    link.rel = "stylesheet";
    link.href = `bundle${index}.css`;
    scope.window.document.head.appendChild(link);
  }
};
