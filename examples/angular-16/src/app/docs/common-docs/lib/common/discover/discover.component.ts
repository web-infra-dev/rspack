import { Component } from '@angular/core';

interface ICompany {
  logo: string;
  name: string;
  description: string;
  link: string;
  scale?: boolean;
};

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'discover',
  templateUrl: './discover.component.html',
  styleUrls: ['discover.component.scss']
})

export class DiscoverComponent {
  name = `Native Angular widgets for Bootstrap 5 and Bootstrap 4 - Discover more`;
  src = 'https://github.com/valor-software/ngx-bootstrap';

  companies: ICompany[] = [
    {
      logo: 'https://www.lumeer.io/wp-content/themes/lumeer/assets/img/logo-big.svg',
      name: 'Lumeer',
      description: 'Easy visual tool for project and team management, that covers all your needs',
      link: 'https://www.lumeer.io/'
    },
    {
      logo: 'https://user-images.githubusercontent.com/663563/70622944-3eb68600-1c1d-11ea-82d3-7ccbd75befe8.png',
      name: 'Sustainablebuildings',
      description: 'SaaS product that give users insight in their power, gas consumption and other data',
      link: 'https://sustainablebuildings.nl'
    },
    {
      logo: 'https://dxc.scene7.com/is/image/dxc/DellTech_1050x1050?qlt=90&wid=1800&ts=1637166138311&$square_desktop$&dpr=off',
      name: 'Dell EMC US',
      description: 'Data Storage, Cloud, Converged and Data Protection',
      link: 'https://www.dellemc.com/en-us/index.htm',
      scale: true
    },
    {
      logo: 'https://upload.wikimedia.org/wikipedia/commons/thumb/9/9d/Soci%C3%A9t%C3%A9_G%C3%A9n%C3%A9rale.svg/279px-Soci%C3%A9t%C3%A9_G%C3%A9n%C3%A9rale.svg.png',
      name: 'Societe Generale Group',
      description: 'The Bank serving 31 million clients worldwide',
      link: 'https://www.societegenerale.com/en/home'
    },
    {
      logo: 'http://www.azerothcore.org/images/logo.png',
      name: 'AzerothCore',
      description: 'AzerothCore, a no-profit organisation composed by volunteers who have a passion for open source',
      link: 'http://www.azerothcore.org/'
    },
    {
      logo: 'https://static.developerhub.io/landing/images/logo-92a931e8f4120d8a7246ea0247865ab5.svg',
      name: 'DeveloperHub',
      description: 'DeveloperHub.io - Hosted Documentation Portals for Product And API Docs',
      link: 'https://developerhub.io',
    },
    {
      logo: 'https://www.inmopc.com/img/products/acuerdos-trovimap.jpg',
      name: 'Trovimap',
      description: 'Trovimap - Houses / Apartments / Locals for sale or rent',
      link: 'https://trovimap.com/'
    },
    {
      logo: 'https://www.atmetis.nl/wp-content/uploads/2017/08/cropped-logo-atmetis-tagline-rgb1500px-1.png',
      name: 'AtMetis',
      description: 'Internal web-application for AtMetis - assessment company from Netherlands',
      link: 'https://www.atmetis.nl/'
    },
    {
      logo: 'https://www.pramati.com/wp-content/themes/twentynineteen-child/images/logo.svg',
      name: 'Pramati Technologies Private Limited',
      description: 'Pramati - Independent, innovative technology companies focused on\n' +
        'profitable, well-defined markets',
      link: 'https://www.pramati.com/'
    },
    {
      logo: 'https://res.cloudinary.com/crunchbase-production/image/upload/c_lpad,h_170,w_170,f_auto,b_white,q_auto:eco/v1484678055/ectxiezxgzm3srv2jkvh.png',
      name: 'Employes',
      description: 'Employes is a dutch based payroll-provider that offers a unique modern way to pay employees.\n',
      link: 'https://app.employes.nl/'
    },
    {
      logo: 'https://easi.net/sites/default/files/styles/gallery_image/public/2019-05/EASI-color.png',
      name: 'EASI',
      description: 'EASI is a cloud, security & software provider offering management applications, professional cloud & security solutions and IT infrastructure services.',
      link: 'https://easi.net/en'
    }
  ];
}
