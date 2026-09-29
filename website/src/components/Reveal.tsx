import React, { useRef } from 'react';
import { useIntersectionObserver } from '../hooks/useIntersectionObserver';

interface RevealProps {
  children: React.ReactNode;
  className?: string;
  delayMs?: number;
  direction?: 'up' | 'down' | 'none';
}

export const Reveal: React.FC<RevealProps> = ({
  children,
  className = '',
  delayMs = 0,
  direction = 'up',
}) => {
  const ref = useRef<HTMLDivElement>(null);
  const isVisible = useIntersectionObserver(ref, { threshold: 0.1, triggerOnce: true });

  const getTransform = () => {
    if (isVisible) return 'translateY(0) scale(1)';
    if (direction === 'up') return 'translateY(16px) scale(0.99)';
    if (direction === 'down') return 'translateY(-16px) scale(0.99)';
    return 'none';
  };

  return (
    <div
      ref={ref}
      style={{
        opacity: isVisible ? 1 : 0,
        transform: getTransform(),
        transition: `opacity 0.5s cubic-bezier(0.16, 1, 0.3, 1) ${delayMs}ms, transform 0.5s cubic-bezier(0.16, 1, 0.3, 1) ${delayMs}ms`,
      }}
      className={className}
    >
      {children}
    </div>
  );
};
