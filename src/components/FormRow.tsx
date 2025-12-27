interface FormRowProps {
  children: React.ReactNode
  className?: string
  variant?: 'horizontal' | 'vertical'
}

export function FormRow({ children, className = '', variant = 'horizontal' }: FormRowProps) {
  const baseClasses = "flex gap-2 sm:gap-3"
  
  const variantClasses = {
    horizontal: "flex-col sm:flex-row",
    vertical: "flex-col space-y-2 sm:space-y-0 sm:flex sm:gap-2"
  }
  
  return (
    <div className={`${baseClasses} ${variantClasses[variant]} ${className}`}>
      {children}
    </div>
  )
}
