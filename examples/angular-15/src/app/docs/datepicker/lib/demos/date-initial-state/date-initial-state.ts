import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-datepicker-date-initial-state',
  templateUrl: './date-initial-state.html'
})
export class DemoDatepickerDateInitialStateComponent {
  bsValue = new Date();
  bsRangeValue: Date[];
  maxDate = new Date();
  constructor() {
    this.maxDate.setDate(this.maxDate.getDate() + 7);
    this.bsRangeValue = [this.bsValue, this.maxDate];
  }
}
