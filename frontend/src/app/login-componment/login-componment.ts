import { Component } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { TokenService } from '../token-service/token-service';
import { CommonModule } from '@angular/common';

@Component({
  selector: 'app-login',
  imports: [FormsModule, CommonModule],
  templateUrl: './login-componment.html',
  styleUrl: './login-componment.css',
})
export class LoginComponent {
  constructor(private tokenService: TokenService) {}

  token: string = '';
  error: string | null = null;

  onSubmit() {
    this.error = null;
    this.tokenService.test(this.token).subscribe({
      next: (res) => {
        console.log('Logged in', res);
      },
      error: (err) => {
        console.error(err);
        this.error = 'Invalid token – please try again';
      },
      complete: () => {},
    });
  }
}
