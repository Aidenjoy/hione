import React, { useState } from 'react';

interface AgentIconProps {
  name: string;
  emoji: string;
  className?: string;
  size?: number;
}

export const AgentIcon: React.FC<AgentIconProps> = ({ name, emoji, className, size = 24 }) => {
  const [error, setError] = useState(false);
  
  // Add a simple cache buster to ensure updates are visible
  const iconPath = name === 'tmux' ? '/icons/tool_tmux.png' : `/icons/tool_${name}.png?v=1`;

  if (error || !name) {
    return <span className={className} style={{ fontSize: size }}>{emoji}</span>;
  }

  return (
    <img 
      src={iconPath} 
      alt={name}
      className={className}
      style={{ width: size, height: size, objectFit: 'contain' }}
      onError={() => setError(true)}
    />
  );
};
