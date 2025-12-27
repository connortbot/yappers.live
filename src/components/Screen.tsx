interface ScreenProps {
  children: React.ReactNode;
  centered?: boolean;
  className?: string;
}

export function Screen({ children, centered = false, className = "" }: ScreenProps) {
  const baseClasses = "min-h-screen w-full pl-safe-left pr-safe-right pt-safe-top pb-safe-bottom relative";
  
  const layoutClasses = centered 
    ? "flex flex-col items-center justify-center p-4" 
    : "p-3 sm:p-4 md:p-6";
    
  const containerClasses = centered 
    ? "" 
    : "max-w-sm sm:max-w-md md:max-w-lg lg:max-w-2xl mx-auto";

  return (
    <div className={`${baseClasses} ${layoutClasses} ${className}`}>
      {centered ? (
        <div className="w-full max-w-sm sm:max-w-md md:max-w-lg lg:max-w-xl xl:max-w-2xl mx-auto flex flex-col items-center relative z-10">
          {children}
        </div>
      ) : (
        <div className={`${containerClasses} relative z-10`}>
          {children}
        </div>
      )}
    </div>
  );
}
