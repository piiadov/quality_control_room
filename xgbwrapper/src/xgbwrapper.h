/**
 * @file xgbwrapper.h
 * @brief XGBoost C Wrapper Library for Quality Control Applications
 * 
 * This library provides a simplified C interface to XGBoost for training
 * and inference of regression models, specifically designed for predicting
 * distribution parameters in quality control scenarios.
 * 
 * ## Public API (6 functions)
 * 
 * **Lifecycle:**
 * - `xgbw_init()` / `xgbw_cleanup()` - Initialize and cleanup library
 * 
 * **Training:**
 * - `xgbw_train_eval()` - Train with auto split, evaluation, and UBJSON save
 * 
 * **Inference:**
 * - `xgbw_predict()` - Load model and make predictions
 * 
 * **Errors:**
 * - `xgbw_get_last_error()` - Get detailed error message
 * - `xgbw_status_string()` - Convert status code to string
 * 
 * ## Error Handling
 * 
 * All functions that can fail return an `XGBWrapperStatus` code. On error,
 * call `xgbw_get_last_error()` to retrieve a human-readable error message.
 * 
 * ## Thread Safety
 * 
 * - `xgbw_init()` and `xgbw_cleanup()` are NOT thread-safe
 * - `xgbw_train_eval()` and `xgbw_predict()` are thread-safe after initialization
 * - Each thread should use separate data buffers
 * 
 * ## Example Usage
 * 
 * ```c
 * xgbw_init();
 * 
 * // Train with evaluation
 * float rmse[2];
 * char model_path[256];
 * XGBWrapperStatus status = xgbw_train_eval(
 *     x, y, rows, x_cols, y_cols, 0.7f,
 *     config, len_config, "./models", "my_model",
 *     model_path, sizeof(model_path), rmse
 * );
 * if (status != XGBW_SUCCESS) {
 *     fprintf(stderr, "Error: %s\n", xgbw_get_last_error());
 * }
 * 
 * // Later, predict
 * xgbw_predict(new_data, rows, x_cols, y_cols, model_path, predictions);
 * 
 * xgbw_cleanup();
 * ```
 * 
 * @version 0.4.0
 * @date 2026
 */

#ifndef XGBWRAPPER_H
#define XGBWRAPPER_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ===========================================================================
 * Export/Import Macros
 * ===========================================================================*/
#if defined(_WIN32) || defined(__CYGWIN__)
    #ifdef XGBWRAPPER_EXPORTS
        #define XGBWRAPPER_API __declspec(dllexport)
    #else
        #define XGBWRAPPER_API __declspec(dllimport)
    #endif
#else
    #ifdef XGBWRAPPER_EXPORTS
        #define XGBWRAPPER_API __attribute__((visibility("default")))
    #else
        #define XGBWRAPPER_API
    #endif
#endif

/* ===========================================================================
 * Type Definitions
 * ===========================================================================*/

/**
 * @brief Return codes for wrapper functions.
 * 
 * All functions that can fail return one of these status codes.
 * Use `xgbw_get_last_error()` to get detailed error information.
 */
typedef enum {
    XGBW_SUCCESS = 0,           /**< Operation completed successfully */
    XGBW_ERROR_INVALID_PARAM,   /**< Invalid parameter (NULL pointer, invalid size) */
    XGBW_ERROR_MEMORY,          /**< Memory allocation failed */
    XGBW_ERROR_FILE_IO,         /**< File I/O error (cannot read/write model) */
    XGBW_ERROR_XGBOOST,         /**< XGBoost internal error */
    XGBW_ERROR_NOT_INITIALIZED, /**< Library not initialized */
    XGBW_ERROR_SIZE_MISMATCH,   /**< Output size doesn't match expected */
} XGBWrapperStatus;

/**
 * @brief Key-value pair for XGBoost configuration parameters.
 * 
 * Used to pass hyperparameters to the training function.
 * Both key and value are null-terminated C strings.
 */
typedef struct {
    const char *key;    /**< Parameter name (e.g., "max_depth", "eta") */
    const char *value;  /**< Parameter value as string (e.g., "4", "0.1") */
} KVPair;

/* ===========================================================================
 * Initialization and Cleanup
 * ===========================================================================*/

/**
 * @brief Initialize the xgbwrapper library.
 * 
 * Must be called before any other xgbwrapper functions.
 * NOT thread-safe - call from main thread before spawning threads.
 * 
 * @return XGBW_SUCCESS on success
 */
XGBWRAPPER_API XGBWrapperStatus xgbw_init(void);

/**
 * @brief Clean up library resources.
 * 
 * Call when done using the library. NOT thread-safe.
 */
XGBWRAPPER_API void xgbw_cleanup(void);

/**
 * @brief Get the last error message.
 * 
 * Thread-local storage is used, so each thread has its own error message.
 * 
 * @return Pointer to error message string (valid until next error)
 */
XGBWRAPPER_API const char* xgbw_get_last_error(void);

/**
 * @brief Get human-readable string for status code.
 * 
 * @param status  Status code to convert
 * @return Static string describing the status
 */
XGBWRAPPER_API const char* xgbw_status_string(XGBWrapperStatus status);

/* ===========================================================================
 * Training
 * ===========================================================================*/

/**
 * @brief Train an XGBoost model with automatic train/test split and evaluation.
 * 
 * Performs a complete train/evaluate cycle:
 * 1. Splits data into train/test sets (using train_ratio)
 * 2. Trains an XGBoost model on training data
 * 3. Evaluates on test data and returns RMSE per target column
 * 4. Saves the model in UBJSON format with timestamp suffix
 * 
 * Output filename: {output_dir}/{model_name}_{YYYYMMDD_HHMMSS}.ubj
 * 
 * @param[in]  x                Full feature matrix (row-major, rows × x_cols)
 * @param[in]  y                Full target matrix (row-major, rows × y_cols)
 * @param[in]  rows             Total number of samples
 * @param[in]  x_cols           Number of feature columns
 * @param[in]  y_cols           Number of target columns  
 * @param[in]  train_ratio      Fraction of data for training (e.g., 0.7 for 70%)
 * @param[in]  config           Array of configuration key-value pairs
 * @param[in]  len_config       Number of configuration pairs
 * @param[in]  output_dir       Directory to save the model
 * @param[in]  model_name       Base name for the model
 * @param[out] actual_path_out  Buffer to receive the actual path written (can be NULL)
 * @param[in]  actual_path_size Size of actual_path_out buffer
 * @param[out] rmse_out         Output RMSE for each target column (pre-allocated, size y_cols)
 * 
 * @return XGBW_SUCCESS, or error code on failure
 * 
 * @note The "num_boost_round" config parameter controls boosting iterations (required).
 */
XGBWRAPPER_API XGBWrapperStatus xgbw_train_eval(
    const float* x, const float* y,
    int rows, int x_cols, int y_cols,
    float train_ratio,
    const KVPair* config, int len_config,
    const char* output_dir,
    const char* model_name,
    char* actual_path_out,
    size_t actual_path_size,
    float* rmse_out
);

/* ===========================================================================
 * Inference
 * ===========================================================================*/

/**
 * @brief Load a trained model and make predictions.
 * 
 * Loads a previously trained XGBoost model from file and generates
 * predictions for the input data.
 * 
 * @param[in]  data           Input features (row-major, rows × x_cols)
 * @param[in]  rows           Number of samples to predict
 * @param[in]  x_cols         Number of feature columns
 * @param[in]  y_cols         Expected number of output columns
 * @param[in]  inference_path Path to the saved model file (.ubj or .json)
 * @param[out] pred           Output predictions (rows × y_cols, pre-allocated)
 * 
 * @return XGBW_SUCCESS, or error code on failure
 * 
 * @pre pred must be pre-allocated with size rows × y_cols
 */
XGBWRAPPER_API XGBWrapperStatus xgbw_predict(
    const float* data,
    int rows, int x_cols, int y_cols,
    const char* inference_path,
    float* pred
);

#ifdef __cplusplus
}
#endif

#endif /* XGBWRAPPER_H */
