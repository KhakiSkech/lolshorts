import { render, screen, fireEvent } from '@testing-library/react';
import { PaymentModal } from './PaymentModal';

// Mock Tauri invoke
const mockInvoke = jest.fn();
global.window.__TAURI__.invoke = mockInvoke;

describe('PaymentModal', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('renders payment modal when open', () => {
    render(<PaymentModal open={true} onOpenChange={() => {}} />);

    expect(screen.getByText(/PRO/i)).toBeInTheDocument();
  });

  it('does not render when closed', () => {
    const { container } = render(
      <PaymentModal open={false} onOpenChange={() => {}} />
    );

    // Modal should not be visible
    expect(container.querySelector('[role="dialog"]')).not.toBeInTheDocument();
  });

  it('calls onOpenChange when close button is clicked', () => {
    const handleOpenChange = jest.fn();
    render(<PaymentModal open={true} onOpenChange={handleOpenChange} />);

    // Find and click close button (X icon)
    const closeButton = screen.getByRole('button', { name: /close/i });
    fireEvent.click(closeButton);

    expect(handleOpenChange).toHaveBeenCalledWith(false);
  });

  it('displays monthly and yearly pricing options', () => {
    render(<PaymentModal open={true} onOpenChange={() => {}} />);

    expect(screen.getByText(/monthly/i)).toBeInTheDocument();
    expect(screen.getByText(/yearly/i)).toBeInTheDocument();
  });

  it('displays correct pricing amounts', () => {
    render(<PaymentModal open={true} onOpenChange={() => {}} />);

    // Check for monthly price (9,900 KRW)
    expect(screen.getByText(/9,900/)).toBeInTheDocument();

    // Check for yearly price (99,000 KRW)
    expect(screen.getByText(/99,000/)).toBeInTheDocument();
  });
});
