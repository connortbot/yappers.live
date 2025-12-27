interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant: 'primary' | 'secondary';
  size: 'icon' | 'small' | 'medium' | 'large';
}

export const Button = ({
  children,
  variant,
  size,
  className,
  ...props
}: ButtonProps) => {
  const baseStyles = "bg-white rounded-lg transform transition-all duration-150 active:translate-y-1 touch-manipulation select-none";
  
  const variantStyles = {
    primary: "font-primary text-black bg-background shadow-black border-2 border-black font-bold",
    secondary: "font-secondary text-pencil bg-background shadow-pencil border-2 border-pencil"
  };

  const sizeStyles = {
    icon: "p-2 min-h-[44px] min-w-[44px]",
    small: "px-3 py-2 min-h-[44px]",
    medium: "px-4 py-3 min-h-[48px]",
    large: "px-6 py-4 min-h-[52px]"
  };

  const fontSize = {
    primary: {
      icon: "text-sm",
      small: "text-sm sm:text-base",
      medium: "text-base sm:text-lg md:text-xl",
      large: "text-lg sm:text-xl md:text-2xl"
    },
    secondary: {
      icon: "text-sm",
      small: "text-xs sm:text-sm",
      medium: "text-sm sm:text-base",
      large: "text-base sm:text-lg"
    }
  }

  const shadowStyles = "shadow-[0_4px_0_0] active:shadow-[0_1px_0_0] hover:shadow-[0_3px_0_0] hover:translate-y-0.25 disabled:shadow-[0_2px_0_0] disabled:opacity-50 disabled:cursor-not-allowed";
  
  return (
    <button
      {...props}
      className={`${baseStyles} ${variantStyles[variant]} ${sizeStyles[size]} ${fontSize[variant][size]} ${shadowStyles} ${className || ''}`}
    >
      {children}
    </button>
  );
};
