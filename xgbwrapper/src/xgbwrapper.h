/**
 * @file xgbwrapper.h
 * @brief XGBoost C Wrapper Library for Quality Control Applications
 * 
 * This library provides a simplified C interface to XGBoost for training
 * and inference of regression models, specifically designed for predicting
 * distribution parameters in quality control scenarios.
 * 
 * ## Error Handling
 * 
 * All functions that can fail return an `XGBWrapperStatus` code. On error,
 * call `xgbw_get_last_error()` to retrieve a human-readable error message.
 * 
 * ## Thread Safety
 * 
 * - `xgbw_init()` and `xgbw_cleanup()` are NOT thread-safe
 * - All other functions are thread-safe after initialization
 * - Each thread should use separate data buffers
 * 
 * ## Example Usage
 * 
 * ```c
 * xgbw_init();
 * 
 * XGBWrapperStatus status = xgbw_train(...);
 * if (status != XGBW_SUCCESS) {
 *     fprintf(stderr, "Error: %s\n", xgbw_get_last_error());
 * }
 * 
 * xgbw_cleanup();
 * ```
 * 
 * @version 0.3.0
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
    const char *key;    /**< Parameter name (e.g., "max_depth", "learning_rate") */
    const char *value;  /**< Parameter value as string (e.g., "10", "0.3") */
} KVPair;

/**
 * @brief Optional logging callback function type.
 * 
 * @param level  Log level: 0=ERROR, 1=WARN, 2=INFO, 3=DEBUG
 * @param msg    Log message (null-terminated)
 */
typedef void (*XGBWrapperLogCallback)(int level, const char* msg);

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
 * @brief Set custom logging callback.
 * 
 * @param callback  Function to receive log messages, or NULL to disable
 */
XGBWRAPPER_API void xgbw_set_log_callback(XGBWrapperLogCallback callback);

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
 * Data Manipulation Functions
 * ===========================================================================*/

/**
 * @brief Shuffle an integer array using Fisher-Yates algorithm.
 * 
 * Initializes the array with sequential values [0, 1, ..., n-1] and then
 * performs an in-place shuffle.
 * 
 * @param[out] array  Array to initialize and shuffle (must be pre-allocated)
 * @param[in]  n      Number of elements in the array
 * 
 * @return XGBW_SUCCESS, or XGBW_ERROR_INVALID_PARAM if array is NULL or n <= 0
 */
XGBWRAPPER_API XGBWrapperStatus xgbw_shuffle(int* array, int n);

/**
 * @brief Split data into training and test sets.
 * 
 * Randomly splits the input data arrays into training and test portions
 * while maintaining the correspondence between features (x) and targets (y).
 * 
 * @param[in]  x           Input feature matrix (row-major, rows × x_cols)
 * @param[in]  y           Input target matrix (row-major, rows × y_cols)
 * @param[out] x_train     Training features (rows_train × x_cols)
 * @param[out] y_train     Training targets (rows_train × y_cols)
 * @param[out] x_test      Test features ((rows - rows_train) × x_cols)
 * @param[out] y_test      Test targets ((rows - rows_train) × y_cols)
 * @param[in]  x_cols      Number of feature columns
 * @param[in]  y_cols      Number of target columns
 * @param[in]  rows        Total number of samples
 * @param[in]  rows_train  Number of samples for training set
 * 
 * @return XGBW_SUCCESS, or error code on failure
 * 
 * @pre All output arrays must be pre-allocated with appropriate sizes.
 * @pre 0 < rows_train < rows
 */
XGBWRAPPER_API XGBWrapperStatus xgbw_split_data(
    const float* x, const float* y,
    float* x_train, float* y_train,
    float* x_test, float* y_test,
    int x_cols, int y_cols, int rows, int rows_train
);

/* ===========================================================================
 * Model Training and Prediction
 * ===========================================================================*/

/**
 * @brief Train an XGBoost model and save to file.
 * 
 * Creates and trains an XGBoost booster with the specified configuration,
 * then saves the model to a JSON file for later inference.
 * 
 * @param[in] x              Training features (row-major, rows × x_cols)
 * @param[in] y              Training targets (row-major, rows × y_cols)
 * @param[in] rows           Number of training samples
 * @param[in] x_cols         Number of feature columns
 * @param[in] y_cols         Number of target columns  
 * @param[in] config         Array of configuration key-value pairs
 * @param[in] len_config     Number of configuration pairs
 * @param[in] inference_path Path to save the trained model (JSON format)
 * 
 * @return XGBW_SUCCESS, or error code on failure
 * 
 * @note The "n_estimators" parameter in config controls the number of 
 *       boosting iterations and is required.
 */
XGBWRAPPER_API XGBWrapperStatus xgbw_train(
    const float* x, const float* y,
    int rows, int x_cols, int y_cols,
    const KVPair* config, int len_config,
    const char* inference_path
);

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
 * @param[in]  inference_path Path to the saved model file
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

/* ===========================================================================
 * Metrics and Utilities
 * ===========================================================================*/

/**
 * @brief Calculate RMSE (Root Mean Square Error) for each target column.
 * 
 * Computes:
 *   RMSE_j = sqrt( (1/n) * sum_i (y_pred[i,j] - y_test[i,j])^2 )
 * 
 * @param[in]  y_pred  Predicted values (row-major, rows × y_cols)
 * @param[in]  y_test  Actual values (row-major, rows × y_cols)
 * @param[in]  rows    Number of samples
 * @param[in]  y_cols  Number of target columns
 * @param[out] rmse    Output RMSE for each column (pre-allocated, size y_cols)
 * 
 * @return XGBW_SUCCESS, or error code on failure
 */
XGBWRAPPER_API XGBWrapperStatus xgbw_calculate_rmse(
    const float* y_pred, const float* y_test,
    int rows, int y_cols,
    float* rmse
);

/**
 * @brief Generate synthetic test data with known relationships.
 * 
 * Generates random features x ∈ [0, 1] and computes targets:
 *   y[0] = sum(x)      (sum of features)
 *   y[1] = sum(sqrt(x)) (sum of square roots)
 * 
 * Useful for testing the training pipeline.
 * 
 * @param[out] x       Feature matrix (rows × x_cols, pre-allocated)
 * @param[out] y       Target matrix (rows × 2, pre-allocated)
 * @param[in]  rows    Number of samples to generate
 * @param[in]  x_cols  Number of features per sample
 * 
 * @return XGBW_SUCCESS, or error code on failure
 */
XGBWRAPPER_API XGBWrapperStatus xgbw_generate_test_data(
    float* x, float* y, int rows, int x_cols
);

#ifdef __cplusplus
}
#endif

#endif /* XGBWRAPPER_H */
