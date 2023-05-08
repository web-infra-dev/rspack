import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-pagination-pages-count-changed',
  templateUrl: './pages-count-changed.html'
})
export class DemoPaginationPagesCountChangedComponent {
  currentPage = 4;
  smallnumPages = 0;
}
