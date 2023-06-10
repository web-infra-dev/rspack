import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-pagination-limit',
  templateUrl: './limit.html'
})
export class DemoPaginationLimitComponent {
  maxSize = 5;
  bigTotalItems = 175;
  bigCurrentPage = 1;
}
