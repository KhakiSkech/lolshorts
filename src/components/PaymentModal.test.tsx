import { render, screen, fireEvent } from '@testing-library/react';
import { PaymentModal } from './PaymentModal';

// Mock shell plugin first
jest.mock('@tauri-apps/plugin-shell', () => ({
  open: jest.fn(),
}));

// Mock Tauri invoke after shell plugin
const mockInvoke = jest.fn();
global.window.__TAURI__ = {
  invoke: mockInvoke,
};

describe('PaymentModal', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('renders payment modal when open', () => {
    render(<PaymentModal isOpen={true} onClose={() => {}} />);

    expect(screen.getByText('Upgrade to LoLShorts PRO')).toBeInTheDocument();
    expect(screen.getByText('PRO Features:')).toBeInTheDocument();
  });

  it('does not render when closed', () => {
    const { container } = render(
      <PaymentModal isOpen={false} onClose={() => {}} />
    );

    // Modal should not be visible
    expect(container.querySelector('[role="dialog"]')).not.toBeInTheDocument();
  });

  it('calls onClose when close button is clicked', () => {
    const handleClose = jest.fn();
    render(<PaymentModal isOpen={true} onClose={handleClose} />);

    // Find and click close button (Cancel button)
    const closeButton = screen.getByText('Cancel');
    fireEvent.click(closeButton);

    expect(handleClose).toHaveBeenCalled();
  });

  it('displays monthly and yearly pricing options', () => {
    render(<PaymentModal isOpen={true} onClose={() => {}} />);

    expect(screen.getByText('Monthly Plan')).toBeInTheDocument();
    expect(screen.getByText('Yearly Plan')).toBeInTheDocument();
  });

  it('displays correct pricing amounts with Korean Won', () => {
    render(<PaymentModal isOpen={true} onClose={() => {}} />);

    // Check for monthly price (₩9,900/month)
    expect(screen.getByText('₩9,900/month')).toBeInTheDocument();

    // Check for yearly price (₩99,000/year)
    expect(screen.getByText('₩99,000/year')).toBeInTheDocument();

    // Check for strikethrough original price
    expect(screen.getByText('₩118,800')).toBeInTheDocument();
  });

  it('shows continue to payment button', () => {
    render(<PaymentModal isOpen={true} onClose={() => {}} />);

    expect(screen.getByText('Continue to Payment')).toBeInTheDocument();
  });
});
