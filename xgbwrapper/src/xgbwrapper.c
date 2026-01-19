/**
 * @file xgbwrapper.c
 * @brief XGBoost C Wrapper Implementation
 * @version 0.2.0
 */

#include "xgbwrapper.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <math.h>
#include <stdarg.h>
#include <xgboost/c_api.h>

/* ===========================================================================
 * Platform-specific Thread-Local Storage
 * ===========================================================================*/
#if defined(_WIN32)
    #define XGBW_THREAD_LOCAL __declspec(thread)
#elif defined(__GNUC__) || defined(__clang__)
    #define XGBW_THREAD_LOCAL __thread
#else
    #define XGBW_THREAD_LOCAL  /* fallback: not thread-safe */
#endif

/* ===========================================================================
 * Internal State
 * ===========================================================================*/

/* Library initialization state */
static int g_initialized = 0;
static unsigned int g_rng_seed = 0;

/* Thread-local error message buffer */
#define XGBW_ERROR_BUF_SIZE 512
static XGBW_THREAD_LOCAL char g_error_buffer[XGBW_ERROR_BUF_SIZE] = {0};

/* Optional logging callback */
static XGBWrapperLogCallback g_log_callback = NULL;

/* Log levels */
#define XGBW_LOG_ERROR 0
#define XGBW_LOG_WARN  1
#define XGBW_LOG_INFO  2
#define XGBW_LOG_DEBUG 3

/* ===========================================================================
 * Validation Macros (DRY principle)
 * ===========================================================================*/

#define XGBW_CHECK_NULL(ptr, name) \
    do { \
        if ((ptr) == NULL) { \
            xgbw_set_error("%s: %s is NULL", __func__, (name)); \
            return XGBW_ERROR_INVALID_PARAM; \
        } \
    } while (0)

#define XGBW_CHECK_POSITIVE(val, name) \
    do { \
        if ((val) <= 0) { \
            xgbw_set_error("%s: %s must be > 0 (got %d)", __func__, (name), (int)(val)); \
            return XGBW_ERROR_INVALID_PARAM; \
        } \
    } while (0)

#define XGBW_CHECK_RANGE(val, min_val, max_val, name) \
    do { \
        if ((val) <= (min_val) || (val) >= (max_val)) { \
            xgbw_set_error("%s: %s must be in (%d, %d) (got %d)", \
                           __func__, (name), (int)(min_val), (int)(max_val), (int)(val)); \
            return XGBW_ERROR_INVALID_PARAM; \
        } \
    } while (0)

#define XGBW_CHECK_STRING(str, name) \
    do { \
        if ((str) == NULL || (str)[0] == '\0') { \
            xgbw_set_error("%s: %s is NULL or empty", __func__, (name)); \
            return XGBW_ERROR_INVALID_PARAM; \
        } \
    } while (0)

/* ===========================================================================
 * Internal Helper Functions
 * ===========================================================================*/

static void xgbw_set_error(const char* fmt, ...) {
    va_list args;
    va_start(args, fmt);
    vsnprintf(g_error_buffer, XGBW_ERROR_BUF_SIZE, fmt, args);
    va_end(args);
    
    if (g_log_callback) {
        g_log_callback(XGBW_LOG_ERROR, g_error_buffer);
    }
}

static void xgbw_log(int level, const char* fmt, ...) {
    if (g_log_callback) {
        char buffer[XGBW_ERROR_BUF_SIZE];
        va_list args;
        va_start(args, fmt);
        vsnprintf(buffer, XGBW_ERROR_BUF_SIZE, fmt, args);
        va_end(args);
        g_log_callback(level, buffer);
    }
}

/* Thread-safe random number generator (simple LCG per-call) */
static unsigned int xgbw_rand(void) {
    /* Using a simple approach: mix time and thread-local state */
    static XGBW_THREAD_LOCAL unsigned int state = 0;
    if (state == 0) {
        state = g_rng_seed ^ (unsigned int)(size_t)&state;
    }
    state = state * 1103515245u + 12345u;
    return (state >> 16) & 0x7fff;
}

#define XGBW_RAND_MAX 0x7fff

/* ===========================================================================
 * Initialization and Cleanup
 * ===========================================================================*/

XGBWrapperStatus xgbw_init(void) {
    if (g_initialized) {
        return XGBW_SUCCESS;  /* Already initialized */
    }
    
    /* Initialize RNG seed */
    g_rng_seed = (unsigned int)time(NULL);
    
    g_initialized = 1;
    xgbw_log(XGBW_LOG_INFO, "xgbwrapper initialized (seed=%u)", g_rng_seed);
    
    return XGBW_SUCCESS;
}

void xgbw_cleanup(void) {
    g_initialized = 0;
    g_log_callback = NULL;
    xgbw_log(XGBW_LOG_INFO, "xgbwrapper cleanup complete");
}

void xgbw_set_log_callback(XGBWrapperLogCallback callback) {
    g_log_callback = callback;
}

const char* xgbw_get_last_error(void) {
    return g_error_buffer;
}

const char* xgbw_status_string(XGBWrapperStatus status) {
    switch (status) {
        case XGBW_SUCCESS:             return "Success";
        case XGBW_ERROR_INVALID_PARAM: return "Invalid parameter";
        case XGBW_ERROR_MEMORY:        return "Memory allocation failed";
        case XGBW_ERROR_FILE_IO:       return "File I/O error";
        case XGBW_ERROR_XGBOOST:       return "XGBoost error";
        case XGBW_ERROR_NOT_INITIALIZED: return "Library not initialized";
        case XGBW_ERROR_SIZE_MISMATCH: return "Size mismatch";
        default:                       return "Unknown error";
    }
}

/* ===========================================================================
 * New Production API Implementation
 * ===========================================================================*/

XGBWrapperStatus xgbw_shuffle(int* array, int n) {
    XGBW_CHECK_NULL(array, "array");
    XGBW_CHECK_POSITIVE(n, "n");
    
    /* Initialize with sequential values */
    for (int i = 0; i < n; ++i) {
        array[i] = i;
    }
    
    /* Fisher-Yates shuffle */
    for (int i = n - 1; i > 0; --i) {
        int j = xgbw_rand() % (i + 1);
        int temp = array[i];
        array[i] = array[j];
        array[j] = temp;
    }
    
    return XGBW_SUCCESS;
}

XGBWrapperStatus xgbw_split_data(
    const float* x, const float* y,
    float* x_train, float* y_train,
    float* x_test, float* y_test,
    int x_cols, int y_cols, int rows, int rows_train
) {
    XGBW_CHECK_NULL(x, "x");
    XGBW_CHECK_NULL(y, "y");
    XGBW_CHECK_NULL(x_train, "x_train");
    XGBW_CHECK_NULL(y_train, "y_train");
    XGBW_CHECK_NULL(x_test, "x_test");
    XGBW_CHECK_NULL(y_test, "y_test");
    XGBW_CHECK_POSITIVE(x_cols, "x_cols");
    XGBW_CHECK_POSITIVE(y_cols, "y_cols");
    XGBW_CHECK_RANGE(rows_train, 0, rows, "rows_train");

    int* indices = (int*)malloc((size_t)rows * sizeof(int));
    if (indices == NULL) {
        xgbw_set_error("split_data: failed to allocate indices array");
        return XGBW_ERROR_MEMORY;
    }
    
    XGBWrapperStatus status = xgbw_shuffle(indices, rows);
    if (status != XGBW_SUCCESS) {
        free(indices);
        return status;
    }

    /* Copy training data */
    for (int i = 0; i < rows_train; ++i) {
        int src_idx = indices[i];
        for (int j = 0; j < x_cols; ++j) {
            x_train[i * x_cols + j] = x[src_idx * x_cols + j];
        }
        for (int j = 0; j < y_cols; ++j) {
            y_train[i * y_cols + j] = y[src_idx * y_cols + j];
        }
    }

    /* Copy test data */
    int rows_test = rows - rows_train;
    for (int i = 0; i < rows_test; ++i) {
        int src_idx = indices[rows_train + i];
        for (int j = 0; j < x_cols; ++j) {
            x_test[i * x_cols + j] = x[src_idx * x_cols + j];
        }
        for (int j = 0; j < y_cols; ++j) {
            y_test[i * y_cols + j] = y[src_idx * y_cols + j];
        }
    }
    
    free(indices);
    return XGBW_SUCCESS;
}

XGBWrapperStatus xgbw_train(
    const float* x, const float* y,
    int rows, int x_cols, int y_cols,
    const KVPair* config, int len_config,
    const char* inference_path
) {
    XGBW_CHECK_NULL(x, "x");
    XGBW_CHECK_NULL(y, "y");
    XGBW_CHECK_NULL(config, "config");
    XGBW_CHECK_STRING(inference_path, "inference_path");
    XGBW_CHECK_POSITIVE(rows, "rows");
    XGBW_CHECK_POSITIVE(x_cols, "x_cols");
    XGBW_CHECK_POSITIVE(y_cols, "y_cols");
    XGBW_CHECK_POSITIVE(len_config, "len_config");

    int status;
    DMatrixHandle dtrain = NULL;
    BoosterHandle booster = NULL;
    XGBWrapperStatus result = XGBW_SUCCESS;

    /* Create DMatrix */
    status = XGDMatrixCreateFromMat(x, (bst_ulong)rows, (bst_ulong)x_cols, -1.0f, &dtrain);
    if (status != 0) {
        xgbw_set_error("train: XGDMatrixCreateFromMat failed: %s", XGBGetLastError());
        return XGBW_ERROR_XGBOOST;
    }

    /* Set labels */
    status = XGDMatrixSetFloatInfo(dtrain, "label", y, (bst_ulong)(rows * y_cols));
    if (status != 0) {
        xgbw_set_error("train: XGDMatrixSetFloatInfo failed: %s", XGBGetLastError());
        result = XGBW_ERROR_XGBOOST;
        goto cleanup;
    }

    /* Create booster */
    status = XGBoosterCreate(&dtrain, 1, &booster);
    if (status != 0) {
        xgbw_set_error("train: XGBoosterCreate failed: %s", XGBGetLastError());
        result = XGBW_ERROR_XGBOOST;
        goto cleanup;
    }

    /* Parse config and set parameters */
    int n_estimators = 0;
    for (int i = 0; i < len_config; ++i) {
        if (config[i].key == NULL || config[i].value == NULL) continue;
        
        if (strcmp(config[i].key, "n_estimators") == 0) {
            n_estimators = atoi(config[i].value);
            continue;
        }
        
        status = XGBoosterSetParam(booster, config[i].key, config[i].value);
        if (status != 0) {
            xgbw_log(XGBW_LOG_WARN, "train: failed to set param %s=%s", 
                     config[i].key, config[i].value);
        }
    }

    if (n_estimators < 1) {
        xgbw_set_error("train: n_estimators must be >= 1 (got %d)", n_estimators);
        result = XGBW_ERROR_INVALID_PARAM;
        goto cleanup;
    }

    /* Training loop */
    xgbw_log(XGBW_LOG_INFO, "train: starting %d iterations", n_estimators);
    for (int iter = 0; iter < n_estimators; ++iter) {
        status = XGBoosterUpdateOneIter(booster, iter, dtrain);
        if (status != 0) {
            xgbw_set_error("train: iteration %d failed: %s", iter, XGBGetLastError());
            result = XGBW_ERROR_XGBOOST;
            goto cleanup;
        }
    }

    /* Save model */
    status = XGBoosterSaveModel(booster, inference_path);
    if (status != 0) {
        xgbw_set_error("train: failed to save model to %s: %s", inference_path, XGBGetLastError());
        result = XGBW_ERROR_FILE_IO;
        goto cleanup;
    }
    
    xgbw_log(XGBW_LOG_INFO, "train: model saved to %s", inference_path);

cleanup:
    if (booster) XGBoosterFree(booster);
    if (dtrain) XGDMatrixFree(dtrain);
    return result;
}

XGBWrapperStatus xgbw_predict(
    const float* data,
    int rows, int x_cols, int y_cols,
    const char* inference_path,
    float* pred
) {
    XGBW_CHECK_NULL(data, "data");
    XGBW_CHECK_NULL(pred, "pred");
    XGBW_CHECK_STRING(inference_path, "inference_path");
    XGBW_CHECK_POSITIVE(rows, "rows");
    XGBW_CHECK_POSITIVE(x_cols, "x_cols");
    XGBW_CHECK_POSITIVE(y_cols, "y_cols");

    int status;
    DMatrixHandle dmatrix = NULL;
    BoosterHandle booster = NULL;
    XGBWrapperStatus result = XGBW_SUCCESS;

    /* Create DMatrix */
    status = XGDMatrixCreateFromMat(data, (bst_ulong)rows, (bst_ulong)x_cols, -1.0f, &dmatrix);
    if (status != 0) {
        xgbw_set_error("predict: XGDMatrixCreateFromMat failed: %s", XGBGetLastError());
        return XGBW_ERROR_XGBOOST;
    }

    /* Create and load booster */
    status = XGBoosterCreate(NULL, 0, &booster);
    if (status != 0) {
        xgbw_set_error("predict: XGBoosterCreate failed: %s", XGBGetLastError());
        result = XGBW_ERROR_XGBOOST;
        goto cleanup;
    }

    status = XGBoosterLoadModel(booster, inference_path);
    if (status != 0) {
        xgbw_set_error("predict: failed to load model from %s: %s", inference_path, XGBGetLastError());
        result = XGBW_ERROR_FILE_IO;
        goto cleanup;
    }

    /* Make predictions */
    bst_ulong out_len = 0;
    const float* out_result = NULL;
    
    status = XGBoosterPredict(booster, dmatrix, 0, 0, 0, &out_len, &out_result);
    if (status != 0) {
        xgbw_set_error("predict: XGBoosterPredict failed: %s", XGBGetLastError());
        result = XGBW_ERROR_XGBOOST;
        goto cleanup;
    }

    /* Validate output size */
    bst_ulong expected_len = (bst_ulong)(y_cols * rows);
    if (out_len != expected_len) {
        xgbw_set_error("predict: size mismatch (expected %lu, got %lu)", 
                       (unsigned long)expected_len, (unsigned long)out_len);
        result = XGBW_ERROR_SIZE_MISMATCH;
        goto cleanup;
    }

    /* Copy results */
    memcpy(pred, out_result, out_len * sizeof(float));

cleanup:
    if (booster) XGBoosterFree(booster);
    if (dmatrix) XGDMatrixFree(dmatrix);
    return result;
}

XGBWrapperStatus xgbw_calculate_rmse(
    const float* y_pred, const float* y_test,
    int rows, int y_cols,
    float* rmse
) {
    XGBW_CHECK_NULL(y_pred, "y_pred");
    XGBW_CHECK_NULL(y_test, "y_test");
    XGBW_CHECK_NULL(rmse, "rmse");
    XGBW_CHECK_POSITIVE(rows, "rows");
    XGBW_CHECK_POSITIVE(y_cols, "y_cols");
    
    for (int j = 0; j < y_cols; ++j) {
        float sse = 0.0f;
        for (int i = 0; i < rows; ++i) {
            float diff = y_pred[i * y_cols + j] - y_test[i * y_cols + j];
            sse += diff * diff;
        }
        rmse[j] = sqrtf(sse / (float)rows);
    }
    
    return XGBW_SUCCESS;
}

XGBWrapperStatus xgbw_generate_test_data(float* x, float* y, int rows, int x_cols) {
    XGBW_CHECK_NULL(x, "x");
    XGBW_CHECK_NULL(y, "y");
    XGBW_CHECK_POSITIVE(rows, "rows");
    XGBW_CHECK_POSITIVE(x_cols, "x_cols");
    
    /* Generate random features in [0, 1] */
    for (int i = 0; i < rows; ++i) {
        for (int j = 0; j < x_cols; ++j) {
            x[i * x_cols + j] = (float)xgbw_rand() / (float)XGBW_RAND_MAX;
        }
    }

    /* Compute targets: y[0] = sum(x), y[1] = sum(sqrt(x)) */
    const int y_cols = 2;
    for (int i = 0; i < rows; ++i) {
        float sum_x = 0.0f;
        float sum_sqrt_x = 0.0f;
        
        for (int k = 0; k < x_cols; ++k) {
            float val = x[i * x_cols + k];
            sum_x += val;
            sum_sqrt_x += sqrtf(val);
        }
        
        y[i * y_cols + 0] = sum_x;
        y[i * y_cols + 1] = sum_sqrt_x;
    }
    
    return XGBW_SUCCESS;
}