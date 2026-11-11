import React from "react";

export interface SocialIconProps extends React.SVGProps<SVGSVGElement> {
  size?: number | string;
  variant?: "brand" | "monochrome";
}

/**
 * TikTok Authentic Brand Icon (with cyan & red chromatic aberration layering)
 */
export function TikTokIcon({
  size = 18,
  variant = "brand",
  className = "",
  ...props
}: SocialIconProps) {
  if (variant === "monochrome") {
    return (
      <svg
        width={size}
        height={size}
        viewBox="0 0 24 24"
        fill="currentColor"
        className={className}
        {...props}
      >
        <path d="M19.59 6.69a4.83 4.83 0 0 1-3.77-4.25V2h-3.45v13.67a2.89 2.89 0 0 1-2.88 2.89 2.89 2.89 0 0 1-2.89-2.89 2.89 2.89 0 0 1 2.89-2.89c.28 0 .54.04.79.1v-3.5a6.37 6.37 0 0 0-.79-.05A6.34 6.34 0 0 0 3 15.67a6.34 6.34 0 0 0 6.34 6.33c3.5 0 6.34-2.83 6.34-6.33V8.71c1.23.86 2.72 1.38 4.33 1.43v-3.4c-.14-.01-.28-.03-.42-.05z" />
      </svg>
    );
  }

  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      className={className}
      {...props}
    >
      {/* Cyan shadow */}
      <path
        d="M18.8 6.2a4.83 4.83 0 0 1-3.77-4.25V1.5h-3.45v13.67a2.89 2.89 0 0 1-2.88 2.89 2.89 2.89 0 0 1-2.89-2.89 2.89 2.89 0 0 1 2.89-2.89c.28 0 .54.04.79.1v-3.5a6.37 6.37 0 0 0-.79-.05A6.34 6.34 0 0 0 2.2 15.17a6.34 6.34 0 0 0 6.34 6.33c3.5 0 6.34-2.83 6.34-6.33V8.21c1.23.86 2.72 1.38 4.33 1.43v-3.4c-.14-.01-.28-.03-.41-.04z"
        fill="#25F4EE"
        opacity="0.9"
      />
      {/* Red/Magenta shadow */}
      <path
        d="M20.2 7.1a4.83 4.83 0 0 1-3.77-4.25V2.4h-3.45v13.67a2.89 2.89 0 0 1-2.88 2.89 2.89 2.89 0 0 1-2.89-2.89 2.89 2.89 0 0 1 2.89-2.89c.28 0 .54.04.79.1v-3.5a6.37 6.37 0 0 0-.79-.05A6.34 6.34 0 0 0 3.7 16.07a6.34 6.34 0 0 0 6.34 6.33c3.5 0 6.34-2.83 6.34-6.33V9.11c1.23.86 2.72 1.38 4.33 1.43v-3.4c-.14-.01-.28-.03-.5-.04z"
        fill="#FE2C55"
        opacity="0.9"
      />
      {/* Front core white/dark */}
      <path
        d="M19.5 6.65a4.83 4.83 0 0 1-3.77-4.25V1.95h-3.45v13.67a2.89 2.89 0 0 1-2.88 2.89 2.89 2.89 0 0 1-2.89-2.89 2.89 2.89 0 0 1 2.89-2.89c.28 0 .54.04.79.1v-3.5a6.37 6.37 0 0 0-.79-.05A6.34 6.34 0 0 0 3 15.62a6.34 6.34 0 0 0 6.34 6.33c3.5 0 6.34-2.83 6.34-6.33V8.66c1.23.86 2.72 1.38 4.33 1.43v-3.4c-.14-.01-.28-.03-.5-.04z"
        fill="#FFFFFF"
      />
    </svg>
  );
}

/**
 * YouTube Authentic Brand Icon (Red pill badge with play triangle)
 */
export function YouTubeIcon({
  size = 18,
  variant = "brand",
  className = "",
  ...props
}: SocialIconProps) {
  if (variant === "monochrome") {
    return (
      <svg
        width={size}
        height={size}
        viewBox="0 0 24 24"
        fill="currentColor"
        className={className}
        {...props}
      >
        <path d="M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.015 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12l-6.273 3.568z" />
      </svg>
    );
  }

  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      className={className}
      {...props}
    >
      <path
        d="M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.015 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814z"
        fill="#FF0000"
      />
      <polygon points="9.545 15.568 15.818 12 9.545 8.432" fill="#FFFFFF" />
    </svg>
  );
}

/**
 * Facebook Official Brand Icon
 */
export function FacebookIcon({
  size = 18,
  variant = "brand",
  className = "",
  ...props
}: SocialIconProps) {
  if (variant === "monochrome") {
    return (
      <svg
        width={size}
        height={size}
        viewBox="0 0 24 24"
        fill="currentColor"
        className={className}
        {...props}
      >
        <path d="M24 12.073c0-6.627-5.373-12-12-12s-12 5.373-12 12c0 5.99 4.388 10.954 10.125 11.854v-8.385H7.078v-3.47h3.047V9.43c0-3.007 1.792-4.669 4.533-4.669 1.312 0 2.686.235 2.686.235v2.953H15.83c-1.491 0-1.956.925-1.956 1.874v2.25h3.328l-.532 3.47h-2.796v8.385C19.612 23.027 24 18.062 24 12.073z" />
      </svg>
    );
  }

  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      className={className}
      {...props}
    >
      <circle cx="12" cy="12" r="12" fill="#1877F2" />
      <path
        d="M16.5 12.073h-2.628v8.385h-3.47V12.073H8.355V9.43h2.047V7.556c0-2.029 1.21-3.151 3.06-3.151.886 0 1.813.158 1.813.158v1.993h-1.022c-1.006 0-1.32.624-1.32 1.265V9.43h3.292l-.525 2.643z"
        fill="#FFFFFF"
      />
    </svg>
  );
}

/**
 * Instagram Gradient Brand Icon
 */
export function InstagramIcon({
  size = 18,
  variant = "brand",
  className = "",
  ...props
}: SocialIconProps) {
  const gradientId = React.useId();

  if (variant === "monochrome") {
    return (
      <svg
        width={size}
        height={size}
        viewBox="0 0 24 24"
        fill="currentColor"
        className={className}
        {...props}
      >
        <path d="M12 2.163c3.204 0 3.584.012 4.85.07 3.252.148 4.771 1.691 4.919 4.919.058 1.265.069 1.645.069 4.849 0 3.205-.012 3.584-.069 4.849-.149 3.225-1.664 4.771-4.919 4.919-1.266.058-1.644.07-4.85.07-3.204 0-3.584-.012-4.849-.07-3.26-.149-4.771-1.699-4.919-4.92-.058-1.265-.07-1.644-.07-4.849 0-3.204.013-3.583.07-4.849.149-3.227 1.664-4.771 4.919-4.919 1.266-.057 1.645-.069 4.849-.069zm0-2.163c-3.259 0-3.667.014-4.947.072-4.358.2-6.78 2.618-6.98 6.98-.059 1.281-.073 1.689-.073 4.948 0 3.259.014 3.668.072 4.948.2 4.358 2.618 6.78 6.98 6.98 1.281.058 1.689.072 4.948.072 3.259 0 3.668-.014 4.948-.072 4.354-.2 6.782-2.618 6.979-6.98.059-1.28.073-1.689.073-4.948 0-3.259-.014-3.667-.072-4.947-.196-4.354-2.617-6.78-6.979-6.98-1.281-.059-1.69-.073-4.949-.073zm0 5.838c-3.403 0-6.162 2.759-6.162 6.162s2.759 6.163 6.162 6.163 6.162-2.759 6.162-6.163c0-3.403-2.759-6.162-6.162-6.162zm0 10.162c-2.209 0-4-1.79-4-4 0-2.209 1.791-4 4-4s4 1.791 4 4c0 2.21-1.791 4-4 4zm6.406-11.845c-.796 0-1.441.645-1.441 1.44s.645 1.44 1.441 1.44c.795 0 1.439-.645 1.439-1.44s-.644-1.44-1.439-1.44z" />
      </svg>
    );
  }

  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      className={className}
      {...props}
    >
      <defs>
        <radialGradient
          id={`${gradientId}-ig-grad`}
          cx="20%"
          cy="110%"
          r="140%"
          fx="20%"
          fy="110%"
        >
          <stop offset="0%" stopColor="#fdf497" />
          <stop offset="5%" stopColor="#fdf497" />
          <stop offset="45%" stopColor="#fd5949" />
          <stop offset="60%" stopColor="#d6249f" />
          <stop offset="90%" stopColor="#285AEB" />
        </radialGradient>
      </defs>
      <rect
        x="2"
        y="2"
        width="20"
        height="20"
        rx="5.5"
        fill={`url(#${gradientId}-ig-grad)`}
      />
      <circle cx="12" cy="12" r="3.7" fill="none" stroke="#FFFFFF" strokeWidth="1.8" />
      <rect
        x="5.5"
        y="5.5"
        width="13"
        height="13"
        rx="3.5"
        fill="none"
        stroke="#FFFFFF"
        strokeWidth="1.8"
      />
      <circle cx="15.8" cy="8.2" r="1" fill="#FFFFFF" />
    </svg>
  );
}

/**
 * Universal Brand Icon component for easy lookup
 */
export function SocialBrandIcon({
  platform,
  size = 18,
  variant = "brand",
  className = "",
  ...props
}: SocialIconProps & { platform: string }) {
  const normalized = platform.toLowerCase();
  switch (normalized) {
    case "tiktok":
      return <TikTokIcon size={size} variant={variant} className={className} {...props} />;
    case "youtube":
    case "shorts":
      return <YouTubeIcon size={size} variant={variant} className={className} {...props} />;
    case "facebook":
      return <FacebookIcon size={size} variant={variant} className={className} {...props} />;
    case "instagram":
    case "reels":
      return <InstagramIcon size={size} variant={variant} className={className} {...props} />;
    default:
      return null;
  }
}
