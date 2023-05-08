import { ActivatedRoute, Route, Router, Routes } from "@angular/router";
import { Component, Inject, Input } from "@angular/core";
import { DOCUMENT } from '@angular/common';
import { DOCS_TOKENS } from '../../tokens/docs-routes-token';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'search-input',
  templateUrl: './search-input.component.html'
})
export class SearchInputComponent {
  @Input() showInput = true;
  isShown = false;
  routes: Routes;
  search = { text: '' };

  constructor(
    private activatedRoute: ActivatedRoute,
    private router: Router,
    @Inject(DOCUMENT) private document: any,
    @Inject(DOCS_TOKENS) _routes: Routes,
  ) {
    this.routes = _routes.filter((v: Route) => v.path !== '**');
  }

  preventReloading(event: KeyboardEvent) {
    if (event.keyCode === 13 || event.key === 'Enter') {
      event.preventDefault();
    }
  }

  getRouteLink(path: string): string {
    const result = this.routes.find(item => item.path === path);
    return result ? `/${path}` : `/components/${path}`;
  }
}


