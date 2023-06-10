import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-datepicker-value-change-event',
  templateUrl: './value-change-event.html'
})
export class DemoDatepickerValueChangeEventComponent {
  data?: Date;

  onValueChange(value: Date): void {
    this.data = value;
  }
}
