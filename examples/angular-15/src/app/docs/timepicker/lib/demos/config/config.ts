import { Component } from '@angular/core';
import { TimepickerConfig } from 'ngx-bootstrap/timepicker';

// such override allows to keep some initial values

export function getTimepickerConfig(): TimepickerConfig {
  return Object.assign(new TimepickerConfig(), {
    hourStep: 2,
    minuteStep: 10,
    showMeridian: false,
    readonlyInput: false,
    mousewheel: true,
    showMinutes: true,
    showSeconds: false,
    labelHours: 'Hours',
    labelMinutes: 'Minutes',
    labelSeconds: 'Seconds'
  });
}

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-timepicker-config',
  templateUrl: './config.html',
  providers: [{ provide: TimepickerConfig, useFactory: getTimepickerConfig }]
})
export class DemoTimepickerConfigComponent {
  mytime?: string;
}
