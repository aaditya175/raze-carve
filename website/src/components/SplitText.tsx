import React, { useRef, useEffect, useState } from 'react';
import { gsap } from 'gsap';
import './SplitText.css';

export interface SplitTextProps {
  text: string;
  className?: string;
  delay?: number;
  duration?: number;
  ease?: string | ((t: number) => number);
  splitType?: 'chars' | 'words' | 'lines' | 'words, chars';
  from?: Record<string, any>;
  to?: Record<string, any>;
  threshold?: number;
  rootMargin?: string;
  tag?: 'h1' | 'h2' | 'h3' | 'h4' | 'h5' | 'h6' | 'p' | 'span';
  textAlign?: React.CSSProperties['textAlign'];
  onLetterAnimationComplete?: () => void;
  showCallback?: boolean;
}

export const SplitText: React.FC<SplitTextProps> = ({
  text,
  className = '',
  delay = 50,
  duration = 1.25,
  ease = 'power3.out',
  splitType = 'chars',
  from = { opacity: 0, y: 40 },
  to = { opacity: 1, y: 0 },
  threshold = 0.1,
  rootMargin = '-100px',
  textAlign = 'left',
  tag = 'h1',
  onLetterAnimationComplete
}) => {
  const containerRef = useRef<HTMLElement>(null);
  const [fontsLoaded, setFontsLoaded] = useState<boolean>(false);

  useEffect(() => {
    if (typeof document !== 'undefined') {
      if (document.fonts && document.fonts.status === 'loaded') {
        setFontsLoaded(true);
      } else if (document.fonts && document.fonts.ready) {
        document.fonts.ready.then(() => setFontsLoaded(true));
      } else {
        setFontsLoaded(true);
      }
    }
  }, []);

  useEffect(() => {
    if (!containerRef.current) return;

    const el = containerRef.current;
    const charElements = el.querySelectorAll('.split-char');
    const wordElements = el.querySelectorAll('.split-word');

    const targets = splitType === 'words' ? wordElements : charElements;

    if (!targets.length) return;

    const ctx = gsap.context(() => {
      gsap.fromTo(
        targets,
        { ...from },
        {
          ...to,
          duration,
          ease,
          stagger: delay / 1000,
          onComplete: () => {
            if (onLetterAnimationComplete) {
              onLetterAnimationComplete();
            }
          }
        }
      );
    }, el);

    return () => ctx.revert();
  }, [text, delay, duration, ease, splitType, from, to, fontsLoaded]);

  // Split text into lines, words, and characters
  const lines = text.split('\n');

  const renderContent = () => {
    return lines.map((line, lineIdx) => {
      const words = line.split(' ');
      return (
        <span key={lineIdx} className="split-line block">
          {words.map((word, wordIdx) => {
            const chars = Array.from(word);
            return (
              <span key={wordIdx} className="split-word inline-block mr-[0.25em]">
                {splitType === 'words' ? (
                  <span className="split-char inline-block">{word}</span>
                ) : (
                  chars.map((char, charIdx) => (
                    <span key={charIdx} className="split-char inline-block">
                      {char}
                    </span>
                  ))
                )}
              </span>
            );
          })}
        </span>
      );
    });
  };

  const Tag = tag as any;

  return (
    <Tag
      ref={containerRef}
      style={{ textAlign, display: 'inline-block' }}
      className={`split-parent ${className}`}
    >
      {renderContent()}
    </Tag>
  );
};

export default SplitText;
