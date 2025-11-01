import React from 'react';

interface SelectProps {
  children: React.ReactNode;
  value: string;
  onChange: (value: string) => void;
  className?: string;
  disabled?: boolean;
  placeholder?: string;
}

export const Select: React.FC<SelectProps> = ({
  children,
  value,
  onChange,
  className = '',
  disabled = false,
  placeholder,
}) => {
  return (
    <select
      value={value}
      onChange={(e) => onChange(e.target.value)}
      disabled={disabled}
      className={`
        block w-full px-3 py-2 border border-gray-300 dark:border-gray-600
        rounded-md shadow-sm bg-white dark:bg-gray-700
        text-gray-900 dark:text-white
        focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500
        disabled:bg-gray-100 dark:disabled:bg-gray-800 disabled:cursor-not-allowed
        ${className}
      `.trim()}
    >
      {placeholder && (
        <option value="" disabled>
          {placeholder}
        </option>
      )}
      {children}
    </select>
  );
};