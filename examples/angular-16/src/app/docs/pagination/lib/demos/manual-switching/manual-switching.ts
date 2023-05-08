import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-pagination-manual-switching',
  templateUrl: './manual-switching.html'
})
export class DemoPaginationManualSwitchingComponent {
  totalItems = 64;
  currentPage = 4;

  setPage(pageNo: number): void {
    this.currentPage = pageNo;
  }
}
