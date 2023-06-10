import { ChangeDetectionStrategy, Component } from '@angular/core';

import { demoComponentContent } from './rating-section.list';
import { ContentSection } from '../../common-docs';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'rating-section',
  templateUrl: './rating-section.component.html',
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class RatingSectionComponent {
  name = 'Rating';
  src = 'https://github.com/valor-software/ngx-bootstrap/tree/development/src/rating';
  componentContent: ContentSection[] = demoComponentContent;
}
