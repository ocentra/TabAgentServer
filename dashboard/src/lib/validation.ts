/**
 * Validation result type
 */
export interface ValidationResult {
  isValid: boolean;
  errors: string[];
}

/**
 * Validation rule type
 */
export type ValidationRule<T> = (value: T) => string | null;

/**
 * Schema validation type
 */
export type ValidationSchema<T> = {
  [K in keyof T]?: ValidationRule<T[K]>[];
};

/**
 * Create a validation rule that checks if value is required
 */
export const required = <T>(message = 'This field is required'): ValidationRule<T> => {
  return (value: T): string | null => {
    if (value === null || value === undefined || value === '') {
      return message;
    }
    return null;
  };
};

/**
 * Create a validation rule that checks string length
 */
export const minLength = (min: number, message?: string): ValidationRule<string> => {
  return (value: string): string | null => {
    if (typeof value === 'string' && value.length < min) {
      return message || `Must be at least ${min} characters long`;
    }
    return null;
  };
};

/**
 * Create a validation rule that checks maximum string length
 */
export const maxLength = (max: number, message?: string): ValidationRule<string> => {
  return (value: string): string | null => {
    if (typeof value === 'string' && value.length > max) {
      return message || `Must be no more than ${max} characters long`;
    }
    return null;
  };
};

/**
 * Create a validation rule that checks if value matches pattern
 */
export const pattern = (regex: RegExp, message?: string): ValidationRule<string> => {
  return (value: string): string | null => {
    if (typeof value === 'string' && !regex.test(value)) {
      return message || 'Invalid format';
    }
    return null;
  };
};

/**
 * Create a validation rule that checks if number is within range
 */
export const numberRange = (min: number, max: number, message?: string): ValidationRule<number> => {
  return (value: number): string | null => {
    if (typeof value === 'number' && (value < min || value > max)) {
      return message || `Must be between ${min} and ${max}`;
    }
    return null;
  };
};

/**
 * Create a validation rule that checks if value is one of allowed values
 */
export const oneOf = <T>(allowedValues: T[], message?: string): ValidationRule<T> => {
  return (value: T): string | null => {
    if (!allowedValues.includes(value)) {
      return message || `Must be one of: ${allowedValues.join(', ')}`;
    }
    return null;
  };
};

/**
 * Create a validation rule that checks if value is a valid email
 */
export const email = (message = 'Invalid email address'): ValidationRule<string> => {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return pattern(emailRegex, message);
};

/**
 * Create a validation rule that checks if value is a valid URL
 */
export const url = (message = 'Invalid URL'): ValidationRule<string> => {
  return (value: string): string | null => {
    try {
      new URL(value);
      return null;
    } catch {
      return message;
    }
  };
};

/**
 * Validate a single value against multiple rules
 */
export function validateValue<T>(value: T, rules: ValidationRule<T>[]): ValidationResult {
  const errors: string[] = [];
  
  for (const rule of rules) {
    const error = rule(value);
    if (error) {
      errors.push(error);
    }
  }
  
  return {
    isValid: errors.length === 0,
    errors,
  };
}

/**
 * Validate an object against a schema
 */
export function validateObject<T extends Record<string, unknown>>(
  obj: T,
  schema: ValidationSchema<T>
): Record<keyof T, ValidationResult> & { isValid: boolean } {
  const results = {} as Record<keyof T, ValidationResult>;
  let isValid = true;
  
  for (const key in schema) {
    const rules = schema[key];
    if (rules) {
      const result = validateValue(obj[key], rules);
      results[key] = result;
      if (!result.isValid) {
        isValid = false;
      }
    }
  }
  
  return { ...results, isValid };
}

/**
 * API response validation schemas
 */
export const apiValidationSchemas = {
  logFilters: {
    level: [oneOf(['debug', 'info', 'warn', 'error', 'all'])],
    source: [minLength(1, 'Source cannot be empty')],
    timeRange: [oneOf(['1h', '6h', '24h', '7d', '30d'])],
  },
  
  modelLoadRequest: {
    model_id: [required('Model ID is required'), minLength(1, 'Model ID cannot be empty')],
  },
  
  configUpdate: {
    // Add specific config validation rules as needed
  },
} as const;

/**
 * Validate API request data
 */
export function validateApiRequest<T extends Record<string, unknown>>(
  data: T,
  schemaName: keyof typeof apiValidationSchemas
): ValidationResult {
  const schema = apiValidationSchemas[schemaName] as ValidationSchema<T>;
  const result = validateObject(data, schema);
  
  return {
    isValid: result.isValid,
    errors: Object.values(result)
      .filter((r): r is ValidationResult => typeof r === 'object' && 'errors' in r)
      .flatMap(r => r.errors),
  };
}