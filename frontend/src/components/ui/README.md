# Paymesh UI Component Library

This directory contains reusable UI components for the Paymesh application. These components are built on top of [Radix UI](https://www.radix-ui.com/) and styled with Tailwind CSS, following the [shadcn/ui](https://ui.shadcn.com/) pattern.

## Importing Components

You can import components directly from `@/components/ui/`:

```tsx
import { Button } from "@/components/ui/button";
// or using the index file
import { Button, Input, toast } from "@/components/ui";
```

## Available Components

### 1. Button

Supports various variants (`primary`, `secondary`, `tertiary`, `destructive`), sizes (`sm`, `md`, `lg`), loading states, and icons.

```tsx
import { Button } from "@/components/ui/button";
import { Mail } from "lucide-react";

// Standard
<Button variant="primary">Click Me</Button>

// Loading
<Button loading>Processing...</Button>

// With Icon
<Button leftIcon={<Mail className="w-4 h-4" />}>Email</Button>
```

### 2. Input

Includes built-in support for labels, helper text, errors, and icons.

```tsx
import { Input } from "@/components/ui/input";
import { Search } from "lucide-react";

// Simple
<Input placeholder="Enter your name" />

// Full
<Input 
  label="Email Address"
  type="email"
  placeholder="john@example.com"
  error="Invalid email" 
  leftIcon={<Search className="w-4 h-4" />}
/>
```

### 3. Checkbox

A controlled or uncontrolled checkbox with label support.

```tsx
import { Checkbox } from "@/components/ui/checkbox";

// Uncontrolled with label
<Checkbox label="Accept terms" />

// Controlled
<Checkbox 
  checked={isChecked} 
  onCheckedChange={setIsChecked} 
  label="Enable notifications" 
/>
```

### 4. Tooltip

A simple wrapper around Radix Tooltip for easy usage.

```tsx
import { Tooltip } from "@/components/ui/tooltip";

<Tooltip content="Add to library" side="top">
  <Button variant="ghost">Hover me</Button>
</Tooltip>
```

### 5. Toast

A toast notification system. usage involves adding the `<Toaster />` component to your layout or page, and using the `toast` function.

**Setup** (usually in root layout):
```tsx
import { Toaster } from "@/components/ui/toaster";

export default function RootLayout({ children }) {
  return (
    <html>
      <body>
        {children}
        <Toaster />
      </body>
    </html>
  );
}
```

**Usage**:
```tsx
import { toast } from "@/components/ui/use-toast";

const handleClick = () => {
  toast({
    title: "Success",
    description: "Your changes have been saved.",
    variant: "success", // or "default", "destructive", "warning", "info"
  });
};
```

## Live Demo

You can view a live demonstration of all these components at:
`http://localhost:3000/test-components`

(See `app/test-components/page.tsx` for the source code)
