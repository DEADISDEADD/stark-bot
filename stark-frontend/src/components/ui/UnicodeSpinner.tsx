import { useEffect, useState } from 'react';
import spinners from 'unicode-animations';
import type { BrailleSpinnerName } from 'unicode-animations';
import clsx from 'clsx';

type SpinnerAnimation = Extract<BrailleSpinnerName, 'rain' | 'pulse' | 'sparkle' | 'orbit'>;

interface UnicodeSpinnerProps {
  animation?: SpinnerAnimation;
  size?: 'sm' | 'md' | 'lg';
  className?: string;
}

const sizeClasses = {
  sm: 'text-sm',
  md: 'text-lg',
  lg: 'text-2xl',
} as const;

export default function UnicodeSpinner({
  animation = 'pulse',
  size = 'md',
  className,
}: UnicodeSpinnerProps) {
  const spinner = spinners[animation];
  const [frame, setFrame] = useState(0);

  useEffect(() => {
    const id = setInterval(() => {
      setFrame((f) => (f + 1) % spinner.frames.length);
    }, spinner.interval);
    return () => clearInterval(id);
  }, [spinner]);

  return (
    <span className={clsx('inline-block font-mono', sizeClasses[size], className)} aria-label="Loading">
      {spinner.frames[frame]}
    </span>
  );
}
