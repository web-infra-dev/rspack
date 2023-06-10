import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-datepicker-datesdisabled',
  templateUrl: './disable-dates.html'
})
export class DemoDatepickerDatesDisabledComponent {
  disabledDates = [
    new Date('2020-02-05'),
    new Date('2020-02-09')
  ];
}
