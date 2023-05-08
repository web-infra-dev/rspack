import { Component } from '@angular/core';
import { PageChangedEvent } from 'ngx-bootstrap/pagination';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-pagination-styling',
  templateUrl: './styling-global.html',
  styles: [
    `
    .btn-custom a {
       background: #31b0d5;
    }
  `
  ]
})
export class DemoPaginationStylingComponent {
  totalItems = 64;
  currentPage = 4;
  smallnumPages = 0;

  pageChanged(event: PageChangedEvent): void {
    console.log('Page changed to: ' + event.page);
    console.log('Number items per page: ' + event.itemsPerPage);
  }
}
