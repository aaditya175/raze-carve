import React, { useCallback, useEffect, useRef } from 'react';
import './CountUp.css';

export interface CountUpProps {
  to: number;
  from?: number;
  direction?: 'up' | 'down';
  delay?: number;
  duration?: number;
  className?: string;
  startWhen?: boolean;
  separator?: string;
  decimals?: number;
  onStart?: () => void;
  onEnd?: () => void;
}

// High-performance quartic ease-out: fast dynamic ramp with an instant, precise stop
const easeOutQuart = (x: number): number => 1 - Math.pow(1 - x, 4);

export const CountUp: React.FC<CountUpProps> = ({
  to,
  from = 0,
  direction = 'up',
  delay = 0,
  duration = 0.7,
  className = '',
  startWhen = true,
  separator = '',
  decimals,
  onStart,
  onEnd,
}) => {
  const ref = useRef<HTMLSpanElement>(null);
  const startVal = direction === 'down' ? to : from;
  const endVal = direction === 'down' ? from : to;

  const getDecimalPlaces = (num: number): number => {
    const str = num.toString();
    if (str.includes('.')) {
      const dec = str.split('.')[1];
      if (parseInt(dec, 10) !== 0) {
        return dec.length;
      }
    }
    return 0;
  };

  const maxDecimals =
    decimals !== undefined ? decimals : Math.max(getDecimalPlaces(from), getDecimalPlaces(to));

  const formatValue = useCallback(
    (latest: number) => {
      const hasDecimals = maxDecimals > 0;

      const options: Intl.NumberFormatOptions = {
        useGrouping: !!separator,
        minimumFractionDigits: hasDecimals ? maxDecimals : 0,
        maximumFractionDigits: hasDecimals ? maxDecimals : 0,
      };

      const formattedNumber = Intl.NumberFormat('en-US', options).format(latest);
      return separator ? formattedNumber.replace(/,/g, separator) : formattedNumber;
    },
    [maxDecimals, separator]
  );

  useEffect(() => {
    const el = ref.current;
    if (!el) return;

    let rafId = 0;
    let timeoutId: ReturnType<typeof setTimeout>;
    let isRunning = false;

    const startAnimation = () => {
      if (isRunning) return;
      isRunning = true;

      timeoutId = setTimeout(() => {
        if (typeof onStart === 'function') onStart();
        const startTime = performance.now();
        const totalDuration = Math.max(duration * 1000, 80);

        const frame = (now: number) => {
          const elapsed = now - startTime;
          const progress = Math.min(elapsed / totalDuration, 1);
          const eased = easeOutQuart(progress);
          const current = startVal + (endVal - startVal) * eased;

          if (el) {
            el.textContent = formatValue(progress >= 1 ? endVal : current);
          }

          if (progress < 1) {
            rafId = requestAnimationFrame(frame);
          } else {
            isRunning = false;
            if (el) el.textContent = formatValue(endVal);
            if (typeof onEnd === 'function') onEnd();
          }
        };

        rafId = requestAnimationFrame(frame);
      }, delay * 1000);
    };

    const reset = () => {
      isRunning = false;
      clearTimeout(timeoutId);
      cancelAnimationFrame(rafId);
      if (el) el.textContent = formatValue(startVal);
    };

    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting && startWhen) {
          startAnimation();
        } else {
          reset();
        }
      },
      { threshold: 0.15 }
    );

    observer.observe(el);

    return () => {
      observer.disconnect();
      clearTimeout(timeoutId);
      cancelAnimationFrame(rafId);
    };
  }, [startVal, endVal, delay, duration, startWhen, formatValue, onStart, onEnd]);

  return (
    <span
      ref={ref}
      className={`count-up-text ${className}`.trim()}
    >
      {formatValue(startVal)}
    </span>
  );
};

export default CountUp;
