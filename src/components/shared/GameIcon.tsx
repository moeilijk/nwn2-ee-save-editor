import type { IconType } from 'react-icons';

interface GameIconProps {
  icon: IconType;
  size?: number;
  color?: string;
  style?: React.CSSProperties;
  className?: string;
}

export function GameIcon({ icon: IconComponent, size = 16, color, style, className }: GameIconProps) {
  return <IconComponent size={size} color={color} style={style} className={className} />;
}
