/**
 * Quality Control Room API Service
 * 
 * WebSocket client for the new split-command server API.
 * Commands: about, analyze, get_intervals, get_cdf, get_pdf, get_histogram
 */

class ApiService {
    constructor(url, timeout = 5000) {
        this.url = url;
        this.timeout = timeout;
        this.socket = null;
        this.pendingRequests = new Map();
        this.requestId = 0;
    }

    /**
     * Connect to WebSocket server
     */
    connect() {
        return new Promise((resolve, reject) => {
            if (this.socket?.readyState === WebSocket.OPEN) {
                resolve({ status: 'connected' });
                return;
            }

            console.log(`Connecting to WebSocket: ${this.url}`);
            this.socket = new WebSocket(this.url);

            this.socket.onopen = () => {
                console.log('WebSocket connected');
                resolve({ status: 'connected' });
            };

            this.socket.onclose = (event) => {
                console.log('WebSocket closed', event.code, event.reason);
                // Reject all pending requests on close
                for (const [command, pending] of this.pendingRequests) {
                    pending.reject(new Error(`Connection closed while waiting for ${command}`));
                }
                this.pendingRequests.clear();
                this.socket = null;
            };

            this.socket.onerror = (error) => {
                console.error('WebSocket error:', error);
                reject({ status: 'error', message: 'Connection failed' });
            };

            this.socket.onmessage = (event) => {
                const response = JSON.parse(event.data);
                console.log('Received:', response.command, response.success, 'pending:', [...this.pendingRequests.keys()]);
                
                // Resolve pending request for this command
                const pending = this.pendingRequests.get(response.command);
                if (pending) {
                    this.pendingRequests.delete(response.command);
                    if (response.success) {
                        pending.resolve(response);
                    } else {
                        pending.reject(new Error(response.message || 'Command failed'));
                    }
                } else {
                    console.warn('No pending request for command:', response.command);
                }
            };
        });
    }

    /**
     * Send a command and wait for response
     */
    send(request) {
        return new Promise((resolve, reject) => {
            if (!this.socket || this.socket.readyState !== WebSocket.OPEN) {
                reject(new Error('Socket not connected'));
                return;
            }

            const command = request.command;
            console.log('Sending:', command);
            
            // Store pending request
            this.pendingRequests.set(command, { resolve, reject });

            // Send request
            this.socket.send(JSON.stringify(request));

            // Timeout
            setTimeout(() => {
                if (this.pendingRequests.has(command)) {
                    console.warn('Timeout for:', command);
                    this.pendingRequests.delete(command);
                    reject(new Error(`Timeout waiting for ${command} response`));
                }
            }, this.timeout);
        });
    }

    /**
     * Close connection
     */
    close() {
        if (this.socket) {
            console.log('Closing WebSocket...');
            this.socket.close();
            this.socket = null;
        }
        this.pendingRequests.clear();
    }

    // =========================================================================
    // API Commands
    // =========================================================================

    /**
     * Get server info
     */
    async about() {
        await this.connect();
        return this.send({ command: 'about' });
    }

    /**
     * Analyze sample data
     * @param {number} distribution - 0=Beta, 1=Normal
     * @param {number[]} data - Raw sample values
     * @param {number} minValue - Domain lower bound
     * @param {number} maxValue - Domain upper bound  
     * @param {number} populationSize - Population size for CI
     */
    async analyze(distribution, data, minValue, maxValue, populationSize) {
        await this.connect();
        return this.send({
            command: 'analyze',
            distribution,
            data,
            min_value: minValue,
            max_value: maxValue,
            population_size: populationSize,
        });
    }

    /**
     * Get confidence interval curves
     * @param {number} distribution - 0=Beta, 1=Normal
     * @param {number[]} scaledData - Scaled sample values from analyze
     * @param {number} populationSize - Population size
     */
    async getIntervals(distribution, scaledData, populationSize) {
        await this.connect();
        return this.send({
            command: 'get_intervals',
            distribution,
            scaled_data: scaledData,
            population_size: populationSize,
        });
    }

    /**
     * Get CDF curves
     * @param {number} distribution - 0=Beta, 1=Normal
     * @param {number[]} paramsMin - CI lower bound params
     * @param {number[]} paramsMax - CI upper bound params
     * @param {number[]} predictedParams - XGBoost predicted params
     * @param {number[]} samplingParams - Method of moments params
     */
    async getCdf(distribution, paramsMin, paramsMax, predictedParams, samplingParams) {
        await this.connect();
        return this.send({
            command: 'get_cdf',
            distribution,
            params_min: paramsMin,
            params_max: paramsMax,
            predicted_params: predictedParams,
            sampling_params: samplingParams,
        });
    }

    /**
     * Get PDF curves
     * @param {number} distribution - 0=Beta, 1=Normal
     * @param {number[]} paramsMin - CI lower bound params
     * @param {number[]} paramsMax - CI upper bound params
     * @param {number[]} predictedParams - XGBoost predicted params
     * @param {number[]} samplingParams - Method of moments params
     */
    async getPdf(distribution, paramsMin, paramsMax, predictedParams, samplingParams) {
        await this.connect();
        return this.send({
            command: 'get_pdf',
            distribution,
            params_min: paramsMin,
            params_max: paramsMax,
            predicted_params: predictedParams,
            sampling_params: samplingParams,
        });
    }

    /**
     * Get histogram data
     * @param {number} distribution - 0=Beta, 1=Normal
     * @param {number[]} scaledData - Scaled sample values
     * @param {number} bins - Number of bins
     * @param {number[]} paramsMin - CI lower bound params
     * @param {number[]} paramsMax - CI upper bound params
     * @param {number[]} predictedParams - XGBoost predicted params
     */
    async getHistogram(distribution, scaledData, bins, paramsMin, paramsMax, predictedParams) {
        await this.connect();
        return this.send({
            command: 'get_histogram',
            distribution,
            scaled_data: scaledData,
            bins,
            params_min: paramsMin,
            params_max: paramsMax,
            predicted_params: predictedParams,
        });
    }

    // =========================================================================
    // Convenience: Full Analysis (calls all commands)
    // =========================================================================

    /**
     * Perform full analysis (analyze + all curves + histogram)
     * This replicates the old monolithic API behavior
     */
    async fullAnalysis(distribution, data, minValue, maxValue, populationSize, bins = 10) {
        try {
            // 1. Analyze
            const analyzeResult = await this.analyze(
                distribution, data, minValue, maxValue, populationSize
            );

            const {
                scaled_data,
                params_min,
                params_max,
                predicted_params,
                sampling_params,
                population_size,
            } = analyzeResult;

            // 2. Get all curves in parallel
            const [intervalsResult, cdfResult, pdfResult, histogramResult] = await Promise.all([
                this.getIntervals(distribution, scaled_data, population_size),
                this.getCdf(distribution, params_min, params_max, predicted_params, sampling_params),
                this.getPdf(distribution, params_min, params_max, predicted_params, sampling_params),
                this.getHistogram(distribution, scaled_data, bins, params_min, params_max, predicted_params),
            ]);

            // Merge all results
            return {
                success: true,
                // From analyze
                sample_size: analyzeResult.sample_size,
                population_size: analyzeResult.population_size,
                min_value: analyzeResult.min_value,
                max_value: analyzeResult.max_value,
                scaled_data,
                params_min,
                params_max,
                predicted_params,
                sampling_params,
                chi2_min: analyzeResult.chi2_min,
                chi2_max: analyzeResult.chi2_max,
                chi2_pred: analyzeResult.chi2_pred,
                // From get_intervals
                cdf_min: intervalsResult.cdf_min,
                cdf_max: intervalsResult.cdf_max,
                // From get_cdf
                domain: cdfResult.domain,
                fitted_cdf_min: cdfResult.fitted_cdf_min,
                fitted_cdf_max: cdfResult.fitted_cdf_max,
                predicted_cdf: cdfResult.predicted_cdf,
                sampling_cdf: cdfResult.sampling_cdf,
                // From get_pdf
                fitted_pdf_min: pdfResult.fitted_pdf_min,
                fitted_pdf_max: pdfResult.fitted_pdf_max,
                predicted_pdf: pdfResult.predicted_pdf,
                sampling_pdf: pdfResult.sampling_pdf,
                // From get_histogram
                bin_edges: histogramResult.bin_edges,
                observed_freq: histogramResult.observed_freq,
                expected_freq_min: histogramResult.expected_freq_min,
                expected_freq_max: histogramResult.expected_freq_max,
                expected_freq_pred: histogramResult.expected_freq_pred,
            };
        } catch (error) {
            return {
                success: false,
                message: error.message,
            };
        }
    }
}

export default ApiService;
