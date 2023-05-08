import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-carousel-interval',
  templateUrl: './interval.html'
})
export class DemoCarouselIntervalComponent {
  myInterval = 1500;
  activeSlideIndex = 0;
  slides: {image: string; text?: string}[] = [
    {image: 'assets/images/nature/3.jpg'},
    {image: 'assets/images/nature/2.jpg'},
    {image: 'assets/images/nature/1.jpg'}
  ];
}
