import { render, screen } from '@testing-library/react';
import { ProtectedFeature } from './ProtectedFeature';
import { useAuthStore } from '@/stores/authStore';

// Mock the auth store
jest.mock('@/stores/authStore');

describe('ProtectedFeature', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('renders children when user is authenticated', () => {
    // Mock authenticated user
    (useAuthStore as unknown as jest.Mock).mockReturnValue({
      user: { id: '123', email: 'test@example.com' },
      isAuthenticated: true,
    });

    render(
      <ProtectedFeature>
        <div>Protected Content</div>
      </ProtectedFeature>
    );

    expect(screen.getByText('Protected Content')).toBeInTheDocument();
  });

  it('renders fallback when user is not authenticated', () => {
    // Mock unauthenticated state
    (useAuthStore as unknown as jest.Mock).mockReturnValue({
      user: null,
      isAuthenticated: false,
    });

    render(
      <ProtectedFeature fallback={<div>Please login</div>}>
        <div>Protected Content</div>
      </ProtectedFeature>
    );

    expect(screen.getByText('Please login')).toBeInTheDocument();
    expect(screen.queryByText('Protected Content')).not.toBeInTheDocument();
  });

  it('renders default fallback when no fallback is provided', () => {
    (useAuthStore as unknown as jest.Mock).mockReturnValue({
      user: null,
      isAuthenticated: false,
    });

    render(
      <ProtectedFeature>
        <div>Protected Content</div>
      </ProtectedFeature>
    );

    // Should render nothing or default message
    expect(screen.queryByText('Protected Content')).not.toBeInTheDocument();
  });
});
