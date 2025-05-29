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
  const baseStyles = "rounded-lg transform transition-all duration-150 active:translate-y-1";
  
  const variantStyles = {
    primary: "font-primary text-black bg-background shadow-black border-2 border-black font-bold",
    secondary: "font-secondary text-pencil bg-background shadow-gray-600 border-2 border-gray-600"
  };

  const sizeStyles = {
    icon: "p-2",
    small: "px-3 py-1.5",
    medium: "px-4 py-2",
    large: "px-6 py-3"
  };

  const fontSize = {
    primary: {
      icon: "text-sm",
      small: "",
      medium: "text-xl",
      large: "text-2xl"
    },
    secondary: {
      icon: "text-sm",
      small: "text-sm",
      medium: "",
      large: "text-lg"
    }
  }

  const shadowStyles = "shadow-[0_4px_0_0] active:shadow-[0_1px_0_0] hover:shadow-[0_3px_0_0] hover:translate-y-0.25";
  
  return (
    <button
      {...props}
      className={`${baseStyles} ${variantStyles[variant]} ${sizeStyles[size]} ${fontSize[variant][size]} ${shadowStyles} ${className || ''}`}
    >
      {children}
    </button>
  );
};
