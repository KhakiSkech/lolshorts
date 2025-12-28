/**
 * Video Modal Component
 *
 * Modal dialog for video playback with proper focus management
 */

import { useState, useEffect, useRef } from 'react';
import { Dialog, DialogContent, DialogOverlay, DialogTitle } from '@/components/ui/dialog';
import { VideoPlayer } from './VideoPlayer';

interface VideoModalProps {
  isOpen: boolean;
  onClose: () => void;
  src: string;
  title?: string;
  autoPlay?: boolean;
}

export function VideoModal({
  isOpen,
  onClose,
  src,
  title,
  autoPlay = false
}: VideoModalProps) {
  const [focusElement, setFocusElement] = useState<HTMLElement | null>(null);
  const overlayRef = useRef<HTMLDivElement>(null);

  // Store the element that had focus before opening the modal
  useEffect(() => {
    if (isOpen && document.activeElement instanceof HTMLElement) {
      setFocusElement(document.activeElement);
    }
  }, [isOpen]);

  // Restore focus when closing
  useEffect(() => {
    if (!isOpen && focusElement) {
      focusElement.focus();
    }
  }, [isOpen, focusElement]);

  // Handle overlay click
  const handleOverlayClick = (e: React.MouseEvent) => {
    if (e.target === overlayRef.current) {
      onClose();
    }
  };

  // Handle keyboard navigation
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (!isOpen) return;

      if (e.key === 'Escape') {
        e.preventDefault();
        onClose();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, onClose]);

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogOverlay
        ref={overlayRef}
        onClick={handleOverlayClick}
        className="fixed inset-0 bg-black/90 backdrop-blur-sm z-50"
      />
      <DialogContent className="fixed inset-4 z-50 bg-background rounded-lg shadow-2xl outline-none max-w-none">
        <DialogTitle className="sr-only">
          {title || 'Video Player'}
        </DialogTitle>
        <div className="h-full flex flex-col">
          <VideoPlayer
            src={src}
            title={title}
            autoPlay={autoPlay}
            onClose={onClose}
            className="flex-1"
          />
        </div>
      </DialogContent>
    </Dialog>
  );
}