import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-prevent-change-to-next-month',
  templateUrl: './prevent-change-to-next-month.component.html'
})
export class DemoDatepickerPreventChangeToNextMonthComponent {
  maxDate = new Date();

  constructor() {
    this.maxDate.setDate(this.maxDate.getDate() + 7);
  }
}
