import { Component } from '@angular/core';
import { FadeInDirective } from '../../shared/fade-in.directive';

@Component({
  selector: 'app-skills',
  imports: [FadeInDirective],
  templateUrl: './skills.html',
  styleUrl: './skills.scss'
})
export class SkillsComponent {}
