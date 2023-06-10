import { Component } from '@angular/core';

import { demoComponentContent } from './carousel-section.list';
import { ContentSection } from '../../common-docs';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'carousel-section',
  templateUrl: './carousel-section.component.html'
})
export class CarouselSectionComponent {
  name = 'Carousel';
  src = 'https://github.com/valor-software/ngx-bootstrap/tree/development/src/carousel';
  componentContent: ContentSection[] = demoComponentContent;
}
