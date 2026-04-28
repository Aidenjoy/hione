import React, { useState } from 'react';
import { cn } from '@/lib/utils';

interface StatusIconProps {
  status: 'online' | 'offline' | 'busy' | 'error' | string;
  className?: string;
  size?: number;
}

export const StatusIcon: React.FC<StatusIconProps> = ({ status, className, size = 16 }) => {
  const [error, setError] = useState(false);
  
  const iconPath = `/icons/status_${status}.png?v=1`;


  if (error || !status) {
    // Fallback to simple colored dot if icon not found
    const colorClass = 
      status === 'online' ? 'bg-green-500' : 
      status === 'busy' ? 'bg-yellow-500' :
      status === 'offline' ? 'bg-gray-400' :
      'bg-red-500';

    return (
      <div 
        className={cn("rounded-full", colorClass, className)} 
        style={{ width: size, height: size }}
      />
    );
  }

  return (
    <img 
      src={iconPath} 
      alt={status}
      className={className}
      style={{ width: size, height: size, objectFit: 'contain' }}
      onError={() => setError(true)}
    />
  );
};
