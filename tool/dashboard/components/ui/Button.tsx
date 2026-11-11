import React, { forwardRef } from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";

const buttonVariants = cva(
  "inline-flex items-center justify-center whitespace-nowrap text-callout font-medium transition-colors duration-150 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:pointer-events-none disabled:opacity-50 select-none",
  {
    variants: {
      variant: {
        primary:
          "bg-primary text-primary-foreground hover:bg-primary-hover",
        secondary:
          "bg-accent text-accent-foreground hover:bg-accent/70",
        outline:
          "border border-border bg-card text-foreground hover:bg-accent/60",
        ghost:
          "text-muted-foreground hover:text-foreground hover:bg-muted/50",
        danger:
          "bg-semantic-danger text-white hover:opacity-90",
      },
      size: {
        xs: "h-6 px-2 rounded-ctl text-caption gap-1",
        sm: "h-6 px-2 rounded-ctl text-footnote gap-1",
        md: "h-7 px-3 rounded-ctl text-footnote gap-2",
        lg: "h-9 px-4 rounded-ctl text-callout gap-2",
        icon: "h-7 w-7 rounded-ctl",
        "icon-sm": "h-6 w-6 rounded-ctl",
        "icon-lg": "h-9 w-9 rounded-ctl",
      },
    },
    defaultVariants: {
      variant: "primary",
      size: "md",
    },
  }
);

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {
  asChild?: boolean;
}

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, size, ...props }, ref) => {
    return (
      <button
        ref={ref}
        className={cn(buttonVariants({ variant, size, className }))}
        {...props}
      />
    );
  }
);

Button.displayName = "Button";
