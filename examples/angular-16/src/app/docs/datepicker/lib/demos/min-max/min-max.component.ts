import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-datepicker-min-max',
  templateUrl: './min-max.component.html'
})
export class DemoDatepickerMinMaxComponent {
  minDate: Date;
  maxDate: Date;

  constructor() {
    this.minDate = new Date();
    this.maxDate = new Date();
    this.minDate.setDate(this.minDate.getDate() - 1);
    this.maxDate.setDate(this.maxDate.getDate() + 7);
  }
}
