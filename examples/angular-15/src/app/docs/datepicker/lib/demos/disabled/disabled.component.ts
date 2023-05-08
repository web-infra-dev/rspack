import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-datepicker-disabled',
  templateUrl: './disabled.component.html'
})
export class DemoDatepickerDisabledComponent {
  isDisabled = false;

  toggleDisabling(): void {
    this.isDisabled = !this.isDisabled;
  }
}
