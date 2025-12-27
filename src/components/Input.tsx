import { forwardRef } from 'react'

interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  variant?: 'default' | 'code'
}

export const Input = forwardRef<HTMLInputElement, InputProps>(
  ({ className = '', variant = 'default', ...props }, ref) => {
    const baseClasses = "border-2 border-pencil text-pencil font-secondary rounded px-2 py-2 sm:py-1 text-base min-h-[48px]"
    
    const variantClasses = {
      default: "w-full",
      code: "w-full sm:w-32 md:w-40 tracking-widest text-center"
    }
    
    return (
      <input
        ref={ref}
        className={`${baseClasses} ${variantClasses[variant]} ${className}`}
        {...props}
      />
    )
  }
)

Input.displayName = 'Input'
