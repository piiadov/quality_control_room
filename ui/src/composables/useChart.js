import { onMounted, onUnmounted, watch } from 'vue';
import { Chart, registerables } from 'chart.js';
import { useThemeStore, useLanguageStore } from '../store';

// Register Chart.js components once
Chart.register(...registerables);

/**
 * Get CSS variable value from document
 * @param {string} varName - CSS variable name (e.g., '--cdf-min-color')
 * @returns {string} The CSS variable value
 */
export function getCssVar(varName) {
    return getComputedStyle(document.documentElement).getPropertyValue(varName).trim();
}

/**
 * Map array data to chart.js {x, y} format
 * @param {number[]} xData - X values
 * @param {number[]} yData - Y values
 * @returns {Array<{x: number, y: number}>}
 */
export function mapToXY(xData, yData) {
    return xData.map((x, i) => ({ x, y: yData[i] }));
}

/**
 * Create a line dataset configuration
 */
export function lineDataset(label, data, colorVar, options = {}) {
    const color = getCssVar(colorVar);
    return {
        type: 'line',
        label,
        data,
        borderColor: color,
        backgroundColor: color,
        borderWidth: 2,
        fill: false,
        pointRadius: 0,
        ...options,
    };
}

/**
 * Create a scatter dataset configuration
 */
export function scatterDataset(label, data, colorVar, options = {}) {
    return {
        type: 'scatter',
        label,
        data,
        borderColor: getCssVar(colorVar),
        pointRadius: 2,
        ...options,
    };
}

/**
 * Composable for chart lifecycle management
 * @param {Ref} chartRef - Vue ref to canvas element
 * @param {Function} createConfig - Function that returns chart config
 * @param {Object} options - Additional options
 * @param {Array} options.watchData - Array of reactive refs to watch for data changes
 * @param {Function} options.onDataChange - Callback when data changes
 * @param {Function} options.onLanguageChange - Callback when language changes
 */
export function useChart(chartRef, createConfig, options = {}) {
    const { watchData = [], onDataChange, onLanguageChange } = options;
    let chartInstance = null;
    
    const theme = useThemeStore();
    const language = useLanguageStore();

    const createChart = () => {
        if (chartRef.value && !chartInstance) {
            chartInstance = new Chart(chartRef.value, createConfig());
        }
    };

    const destroyChart = () => {
        if (chartInstance) {
            chartInstance.destroy();
            chartInstance = null;
        }
    };

    const recreateChart = () => {
        destroyChart();
        createChart();
    };

    onMounted(() => {
        createChart();
    });

    onUnmounted(() => {
        destroyChart();
    });

    // Watch for theme changes - recreate chart
    watch(() => theme.currentTheme, (newTheme, oldTheme) => {
        if (newTheme !== oldTheme && chartInstance) {
            recreateChart();
        }
    });

    // Watch for language changes - update labels
    watch(() => language.currentLanguage, (newLang, oldLang) => {
        if (newLang !== oldLang && chartInstance && onLanguageChange) {
            onLanguageChange(chartInstance);
        }
    });

    // Watch for data changes
    if (watchData.length > 0 && onDataChange) {
        watch(watchData, () => {
            if (chartInstance) {
                onDataChange(chartInstance);
            }
        }, { deep: true });
    }

    return {
        chartRef,
        chartInstance: () => chartInstance,
        recreateChart,
    };
}

/**
 * Default chart options for distribution charts
 */
export function getDefaultChartOptions(title, xLabel = 'x', yLabel = 'y') {
    return {
        responsive: true,
        maintainAspectRatio: false,
        plugins: {
            legend: {
                position: 'bottom',
                labels: {
                    font: { size: 12 },
                },
            },
            title: {
                display: true,
                text: title,
                font: { size: 14 },
            },
        },
        scales: {
            x: {
                type: 'linear',
                title: {
                    display: true,
                    text: xLabel,
                },
                grid: {
                    color: getCssVar('--grid-color'),
                },
            },
            y: {
                title: {
                    display: true,
                    text: yLabel,
                },
                grid: {
                    color: getCssVar('--grid-color'),
                },
            },
        },
    };
}

/**
 * Default chart options for histogram
 */
export function getHistogramOptions(title, xLabel, yLabel) {
    return {
        responsive: true,
        maintainAspectRatio: false,
        plugins: {
            legend: { display: false },
            title: {
                display: true,
                text: title,
                font: { size: 14 },
            },
        },
        scales: {
            x: {
                type: 'category',
                title: {
                    display: true,
                    text: xLabel,
                },
                grid: {
                    color: getCssVar('--grid-color'),
                },
                offset: true,
                barPercentage: 1.0,
                categoryPercentage: 1.0,
            },
            y: {
                title: {
                    display: true,
                    text: yLabel,
                },
                grid: {
                    color: getCssVar('--grid-color'),
                },
                beginAtZero: true,
            },
        },
    };
}
