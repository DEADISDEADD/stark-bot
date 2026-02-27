import { ButtonHTMLAttributes, forwardRef } from 'react';
import clsx from 'clsx';
import UnicodeSpinner from './UnicodeSpinner';

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'danger' | 'ghost';
  size?: 'sm' | 'md' | 'lg';
  isLoading?: boolean;
}

const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant = 'primary', size = 'md', isLoading, disabled, children, ...props }, ref) => {
    const baseStyles =
      'inline-flex items-center justify-center font-semibold rounded-lg transition-all focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-slate-900 disabled:opacity-50 disabled:cursor-not-allowed';

    const variants = {
      primary:
        'bg-gradient-to-r from-stark-500 to-stark-600 hover:from-stark-400 hover:to-stark-500 text-white focus:ring-stark-500 hover:-translate-y-0.5 hover:shadow-lg hover:shadow-stark-500/25 disabled:hover:translate-y-0 disabled:hover:shadow-none',
      secondary:
        'bg-slate-700 hover:bg-slate-600 text-white focus:ring-slate-500',
      danger:
        'bg-red-600 hover:bg-red-500 text-white focus:ring-red-500',
      ghost:
        'bg-transparent hover:bg-slate-700/50 text-slate-400 hover:text-white focus:ring-slate-500',
    };

    const sizes = {
      sm: 'px-3 py-1.5 text-sm',
      md: 'px-4 py-2.5 text-sm',
      lg: 'px-6 py-3 text-base',
    };

    return (
      <button
        ref={ref}
        className={clsx(baseStyles, variants[variant], sizes[size], className)}
        disabled={disabled || isLoading}
        {...props}
      >
        {isLoading ? (
          <>
            <UnicodeSpinner animation="pulse" size="sm" className="-ml-1 mr-2" />
            Loading...
          </>
        ) : (
          children
        )}
      </button>
    );
  }
);

Button.displayName = 'Button';

export default Button;
