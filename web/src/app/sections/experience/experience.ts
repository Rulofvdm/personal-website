import { Component } from '@angular/core';
import { FadeInDirective } from '../../shared/fade-in.directive';

@Component({
  selector: 'app-experience',
  imports: [FadeInDirective],
  templateUrl: './experience.html',
  styleUrl: './experience.scss'
})
export class ExperienceComponent {}
