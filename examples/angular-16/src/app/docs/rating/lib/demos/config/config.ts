import { Component } from '@angular/core';
import { RatingConfig } from 'ngx-bootstrap/rating';

// such override allows to keep some initial values
export function getRatingConfig(): RatingConfig {
  return Object.assign(new RatingConfig(), { ariaLabel: 'My Rating' });
}

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-rating-config',
  templateUrl: './config.html',
  providers: [{ provide: RatingConfig, useFactory: getRatingConfig }]
})
export class DemoRatingConfigComponent {
  max = 10;
  rate = 7;
}
