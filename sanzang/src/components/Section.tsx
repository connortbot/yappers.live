interface SectionProps {
  children: React.ReactNode
  title?: string
  className?: string
  variant?: 'default' | 'error'
}

export function Section({ children, title, className = '', variant = 'default' }: SectionProps) {
  const baseClasses = "bg-white mb-6 sm:mb-8 p-3 sm:p-4 rounded-lg"
  
  const variantClasses = {
    default: "border-2 border-pencil",
    error: "bg-red-100 border-2 border-red-500"
  }
  
  return (
    <div className={`${baseClasses} ${variantClasses[variant]} ${className}`}>
      {title && (
        <h2 className="text-xl sm:text-2xl md:text-3xl font-semibold mb-3 sm:mb-4 font-primary">
          {title}
        </h2>
      )}
      {children}
    </div>
  )
} 