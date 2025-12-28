import { invoke } from '@tauri-apps/api/core';

export interface User {
  id: string;
  email: string;
  tier: 'Free' | 'Pro';
  access_token: string;
  refresh_token: string;
  expires_at: number;
}

export interface LoginCredentials {
  email: string;
  password: string;
}

export interface SignupCredentials {
  email: string;
  password: string;
}

export class DatabaseAuthService {
  async login(credentials: LoginCredentials): Promise<User> {
    try {
      const user = await invoke<User>('db_login', {
        email: credentials.email,
        password: credentials.password,
      });

      // Store in localStorage for persistence
      localStorage.setItem('user', JSON.stringify(user));

      return user;
    } catch (error) {
      console.error('Login failed:', error);
      throw new Error(typeof error === 'string' ? error : 'Login failed');
    }
  }

  async signup(credentials: SignupCredentials): Promise<User> {
    try {
      const user = await invoke<User>('db_signup', {
        email: credentials.email,
        password: credentials.password,
      });

      // Store in localStorage for persistence
      localStorage.setItem('user', JSON.stringify(user));

      return user;
    } catch (error) {
      console.error('Signup failed:', error);
      throw new Error(typeof error === 'string' ? error : 'Signup failed');
    }
  }

  async getCurrentUser(userId: string): Promise<User> {
    try {
      const user = await invoke<User>('db_get_current_user', {
        userId,
      });

      return user;
    } catch (error) {
      console.error('Get current user failed:', error);
      throw new Error(typeof error === 'string' ? error : 'Failed to get user');
    }
  }

  logout() {
    localStorage.removeItem('user');
  }

  getStoredUser(): User | null {
    const userStr = localStorage.getItem('user');
    if (!userStr) return null;

    try {
      return JSON.parse(userStr);
    } catch {
      return null;
    }
  }

  isAuthenticated(): boolean {
    const user = this.getStoredUser();
    if (!user) return false;

    // Check if token is expired
    const now = Math.floor(Date.now() / 1000);
    return user.expires_at > now;
  }
}

export const dbAuth = new DatabaseAuthService();
