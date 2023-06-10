import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'keep-dates-out-of-rules',
  templateUrl: './keep-dates-out-of-rules.component.html'
})
export class KeepDatesOutOfRulesComponent {
  minDate = new Date();
  bsRangeValue: Date[];
  maxDate = new Date();
  bsInvalidDate: Date = new Date();

  constructor() {
    this.maxDate.setDate(this.maxDate.getDate() + 7);
    this.bsInvalidDate.setDate(this.maxDate.getDate() + 2);
    this.bsRangeValue = [this.minDate, this.bsInvalidDate];
  }
}
