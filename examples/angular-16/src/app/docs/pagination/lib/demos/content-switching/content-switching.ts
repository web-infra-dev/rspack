import { Component, OnInit } from '@angular/core';
import { PageChangedEvent } from 'ngx-bootstrap/pagination';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-pagination-content-switching',
  templateUrl: './content-switching.html',
  styles: [`
    .content-wrapper {
      border: 1px solid #ddd;
      border-radius: 4px;
      padding-left: 10px;
      margin-bottom: 10px;
    }
  `]
})
export class DemoPaginationContentSwitchingComponent implements OnInit {
  contentArray = new Array(90).fill('');
  returnedArray?: string[];

  ngOnInit(): void {
    this.contentArray = this.contentArray.map((v: string, i: number) => `Content line ${i + 1}`);
    this.returnedArray = this.contentArray.slice(0, 10);
  }

  pageChanged(event: PageChangedEvent): void {
    const startItem = (event.page - 1) * event.itemsPerPage;
    const endItem = event.page * event.itemsPerPage;
    this.returnedArray = this.contentArray.slice(startItem, endItem);
  }
}
