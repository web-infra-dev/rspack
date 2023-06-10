import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-pagination-disabled',
  templateUrl: './disabled.html'
})
export class DemoPaginationDisabledComponent {
  disabled = false;

  toggleState(): void {
    this.disabled = !this.disabled;
  }
}
