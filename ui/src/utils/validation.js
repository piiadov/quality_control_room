/**
 * Validation utilities for input data
 */

/**
 * Parse a string containing numbers into an array
 * @param {string} str - Input string with numbers
 * @returns {number[]} Array of numbers
 */
export function parseNumberArray(str) {
    if (!str || str === "") return [];
    const matches = str.toString().match(/-?\d*\.?\d+/g);
    return matches ? matches.map(Number) : [];
}

/**
 * Validate population size
 * @param {string|number} value - Input value
 * @returns {{ valid: boolean, value?: number, error?: string }}
 */
export function validatePopulationSize(value) {
    const parsed = parseInt(value, 10);
    if (isNaN(parsed) || parsed < 1) {
        return { 
            valid: false, 
            error: 'Population size: Please enter valid positive integer number' 
        };
    }
    return { valid: true, value: parsed };
}

/**
 * Validate min/max range
 * @param {string|number} minValue - Min value
 * @param {string|number} maxValue - Max value
 * @returns {{ valid: boolean, min?: number, max?: number, error?: string }}
 */
export function validateRange(minValue, maxValue) {
    const min = parseFloat(minValue);
    const max = parseFloat(maxValue);
    
    if (isNaN(min) || isNaN(max)) {
        return { 
            valid: false, 
            error: 'Min or Max value: Please enter valid float number' 
        };
    }
    
    if (min >= max) {
        return { 
            valid: false, 
            error: 'Min value must be less than max value' 
        };
    }
    
    return { valid: true, min, max };
}

/**
 * Validate sampling data
 * @param {string|number[]} data - Raw data input
 * @param {number} minValue - Min allowed value
 * @param {number} maxValue - Max allowed value
 * @param {number} populationSize - Population size
 * @returns {{ valid: boolean, data?: number[], error?: string }}
 */
export function validateSamplingData(data, minValue, maxValue, populationSize) {
    const parsed = Array.isArray(data) ? data : parseNumberArray(data.toString());
    
    if (parsed.length === 0) {
        return { 
            valid: false, 
            error: 'Sampling data: Please enter valid float numbers' 
        };
    }
    
    if (populationSize < parsed.length) {
        return { 
            valid: false, 
            error: 'Population size must be greater than sample size' 
        };
    }
    
    const allInRange = parsed.every(v => v >= minValue && v <= maxValue);
    if (!allInRange) {
        return { 
            valid: false, 
            error: 'Sampling data: All values must be within the specified range' 
        };
    }
    
    return { valid: true, data: parsed };
}

/**
 * Validate bins number
 * @param {string|number} value - Input value
 * @returns {{ valid: boolean, value?: number, error?: string }}
 */
export function validateBinsNumber(value) {
    const parsed = parseInt(value, 10);
    if (isNaN(parsed) || parsed < 1 || parsed > 50) {
        return { 
            valid: false, 
            error: 'Please enter a valid number of bins (1-50)' 
        };
    }
    return { valid: true, value: parsed };
}
