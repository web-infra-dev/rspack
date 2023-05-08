import { Component } from '@angular/core';
import { DatepickerDateCustomClasses } from 'ngx-bootstrap/datepicker';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-datepicker-date-custom-classes',
  templateUrl: './date-custom-classes.html',
  styleUrls: ['./date-custom-classes.scss']
})
export class DemoDatepickerDateCustomClassesComponent {
  dateCustomClasses: DatepickerDateCustomClasses[];

  constructor() {
    const now = new Date();
    const twoDaysAhead = new Date();
    twoDaysAhead.setDate(now.getDate() + 2);
    const fourDaysAhead = new Date();
    fourDaysAhead.setDate(now.getDate() + 4);

    this.dateCustomClasses = [
      { date: now, classes: [] },
      { date: twoDaysAhead, classes: ['bg-warning'] },
      { date: fourDaysAhead, classes: ['bg-danger', 'text-warning'] }
    ];
  }
}
