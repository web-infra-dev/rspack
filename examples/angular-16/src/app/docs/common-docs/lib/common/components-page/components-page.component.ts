import { Component, Inject } from "@angular/core";
import { DOCS_TOKENS } from "../../tokens/docs-routes-token";
import { Routes } from "@angular/router";

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'components',
  templateUrl: './components-page.component.html'
})
export class ComponentsPageComponent {
  componentsArray?:Routes;

  constructor(
    @Inject(DOCS_TOKENS) _routes: Routes
  ) {
    this.componentsArray = _routes.find(route => route.path === "components")?.children?.filter(route => route.path);
  }
}
