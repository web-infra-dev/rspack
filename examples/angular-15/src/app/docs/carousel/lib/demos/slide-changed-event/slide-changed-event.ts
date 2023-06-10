import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-carousel-slide-changed-event',
  templateUrl: './slide-changed-event.html'
})
export class DemoCarouselSlideChangedEventComponent {
  slideChangeMessage = '';

  slides: {image: string; text?: string}[] = [
    {image: 'assets/images/nature/7.jpg'},
    {image: 'assets/images/nature/5.jpg'},
    {image: 'assets/images/nature/3.jpg'}
  ];

  log(event: number) {
    // simple hack for expression has been changed error
    setTimeout(() => {
      this.slideChangeMessage = `Slide has been switched: ${event}`;
    });
  }
}
