import { Component } from '@angular/core';
import { TimepickerConfig } from 'ngx-bootstrap/timepicker';

export function getTimepickerConfig(): TimepickerConfig {
  return Object.assign(new TimepickerConfig(), {
    allowEmptyTime: true
  });
}

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-timepicker-empty-date',
  templateUrl: './empty-date.html',
  providers: [{ provide: TimepickerConfig, useFactory: getTimepickerConfig }]
})
export class DemoTimepickerEmptyDateComponent {
  allowEmptyTime = true;
  myTime?: Date = new Date();
  isValid?: boolean;

  clear(): void {
    this.myTime = void 0;
  }
}
