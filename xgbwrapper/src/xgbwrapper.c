/**
 * @file xgbwrapper.c
 * @brief XGBoost C Wrapper Implementation
 * @version 0.4.0
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

static int g_initialized = 0;
static unsigned int g_rng_seed = 0;

#define XGBW_ERROR_BUF_SIZE 512
static XGBW_THREAD_LOCAL char g_error_buffer[XGBW_ERROR_BUF_SIZE] = {0};

/* ===========================================================================
 * Validation Macros
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
 * Internal Helper Functions (static)
 * ===========================================================================*/

static void xgbw_set_error(const char* fmt, ...) {
    va_list args;
    va_start(args, fmt);
    vsnprintf(g_error_buffer, XGBW_ERROR_BUF_SIZE, fmt, args);
    va_end(args);
}

/* Thread-safe random number generator */
static unsigned int xgbw_rand(void) {
    static XGBW_THREAD_LOCAL unsigned int state = 0;
    if (state == 0) {
        state = g_rng_seed ^ (unsigned int)(size_t)&state;
    }
    state = state * 1103515245u + 12345u;
    return (state >> 16) & 0x7fff;
}

#define XGBW_RAND_MAX 0x7fff

/* Fisher-Yates shuffle (internal) */
static XGBWrapperStatus shuffle_array(int* array, int n) {
    if (array == NULL || n <= 0) {
        return XGBW_ERROR_INVALID_PARAM;
    }
    
    for (int i = 0; i < n; ++i) {
        array[i] = i;
    }
    
    for (int i = n - 1; i > 0; --i) {
        int j = xgbw_rand() % (i + 1);
        int temp = array[i];
        array[i] = array[j];
        array[j] = temp;
    }
    
    return XGBW_SUCCESS;
}

/* Split data into train/test sets (internal) */
static XGBWrapperStatus split_data(
    const float* x, const float* y,
    float* x_train, float* y_train,
    float* x_test, float* y_test,
    int x_cols, int y_cols, int rows, int rows_train
) {
    int* indices = (int*)malloc((size_t)rows * sizeof(int));
    if (indices == NULL) {
        xgbw_set_error("split_data: memory allocation failed");
        return XGBW_ERROR_MEMORY;
    }
    
    XGBWrapperStatus status = shuffle_array(indices, rows);
    if (status != XGBW_SUCCESS) {
        free(indices);
        return status;
    }

    /* Copy training data */
    for (int i = 0; i < rows_train; ++i) {
        int src = indices[i];
        for (int j = 0; j < x_cols; ++j) {
            x_train[i * x_cols + j] = x[src * x_cols + j];
        }
        for (int j = 0; j < y_cols; ++j) {
            y_train[i * y_cols + j] = y[src * y_cols + j];
        }
    }

    /* Copy test data */
    int rows_test = rows - rows_train;
    for (int i = 0; i < rows_test; ++i) {
        int src = indices[rows_train + i];
        for (int j = 0; j < x_cols; ++j) {
            x_test[i * x_cols + j] = x[src * x_cols + j];
        }
        for (int j = 0; j < y_cols; ++j) {
            y_test[i * y_cols + j] = y[src * y_cols + j];
        }
    }
    
    free(indices);
    return XGBW_SUCCESS;
}

/* Calculate RMSE (internal) */
static XGBWrapperStatus calculate_rmse(
    const float* y_pred, const float* y_test,
    int rows, int y_cols,
    float* rmse
) {
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

/* ===========================================================================
 * Public API: Initialization and Cleanup
 * ===========================================================================*/

XGBWrapperStatus xgbw_init(void) {
    if (g_initialized) {
        return XGBW_SUCCESS;
    }
    g_rng_seed = (unsigned int)time(NULL);
    g_initialized = 1;
    return XGBW_SUCCESS;
}

void xgbw_cleanup(void) {
    g_initialized = 0;
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
 * Public API: Training
 * ===========================================================================*/

XGBWrapperStatus xgbw_train_eval(
    const float* x, const float* y,
    int rows, int x_cols, int y_cols,
    float train_ratio,
    const KVPair* config, int len_config,
    const char* output_dir,
    const char* model_name,
    char* actual_path_out,
    size_t actual_path_size,
    float* rmse_out
) {
    XGBW_CHECK_NULL(x, "x");
    XGBW_CHECK_NULL(y, "y");
    XGBW_CHECK_NULL(config, "config");
    XGBW_CHECK_NULL(rmse_out, "rmse_out");
    XGBW_CHECK_STRING(output_dir, "output_dir");
    XGBW_CHECK_STRING(model_name, "model_name");
    XGBW_CHECK_POSITIVE(rows, "rows");
    XGBW_CHECK_POSITIVE(x_cols, "x_cols");
    XGBW_CHECK_POSITIVE(y_cols, "y_cols");
    XGBW_CHECK_POSITIVE(len_config, "len_config");

    if (train_ratio <= 0.0f || train_ratio >= 1.0f) {
        xgbw_set_error("xgbw_train_eval: train_ratio must be in (0, 1), got %f", train_ratio);
        return XGBW_ERROR_INVALID_PARAM;
    }

    int status;
    XGBWrapperStatus result = XGBW_SUCCESS;
    
    /* Allocate split buffers */
    int rows_train = (int)(train_ratio * rows);
    int rows_test = rows - rows_train;
    
    float* x_train = (float*)malloc((size_t)rows_train * x_cols * sizeof(float));
    float* y_train = (float*)malloc((size_t)rows_train * y_cols * sizeof(float));
    float* x_test = (float*)malloc((size_t)rows_test * x_cols * sizeof(float));
    float* y_test = (float*)malloc((size_t)rows_test * y_cols * sizeof(float));
    float* y_pred = (float*)malloc((size_t)rows_test * y_cols * sizeof(float));
    
    if (!x_train || !y_train || !x_test || !y_test || !y_pred) {
        xgbw_set_error("xgbw_train_eval: memory allocation failed");
        result = XGBW_ERROR_MEMORY;
        goto cleanup_buffers;
    }
    
    /* Split data */
    result = split_data(x, y, x_train, y_train, x_test, y_test,
                        x_cols, y_cols, rows, rows_train);
    if (result != XGBW_SUCCESS) {
        goto cleanup_buffers;
    }
    
    /* Generate timestamped filename */
    char timestamp[20];
    time_t now = time(NULL);
    struct tm* tm_info = localtime(&now);
    strftime(timestamp, sizeof(timestamp), "%Y%m%d_%H%M%S", tm_info);

    char full_path[512];
    snprintf(full_path, sizeof(full_path), "%s/%s_%s.ubj", output_dir, model_name, timestamp);

    if (actual_path_out != NULL && actual_path_size > 0) {
        strncpy(actual_path_out, full_path, actual_path_size - 1);
        actual_path_out[actual_path_size - 1] = '\0';
    }
    
    /* Train on training data */
    DMatrixHandle dtrain = NULL;
    DMatrixHandle dtest = NULL;
    BoosterHandle booster = NULL;
    
    status = XGDMatrixCreateFromMat(x_train, (bst_ulong)rows_train, (bst_ulong)x_cols, -1.0f, &dtrain);
    if (status != 0) {
        xgbw_set_error("xgbw_train_eval: XGDMatrixCreateFromMat failed: %s", XGBGetLastError());
        result = XGBW_ERROR_XGBOOST;
        goto cleanup_xgb;
    }

    status = XGDMatrixSetFloatInfo(dtrain, "label", y_train, (bst_ulong)(rows_train * y_cols));
    if (status != 0) {
        xgbw_set_error("xgbw_train_eval: XGDMatrixSetFloatInfo failed: %s", XGBGetLastError());
        result = XGBW_ERROR_XGBOOST;
        goto cleanup_xgb;
    }

    status = XGBoosterCreate(&dtrain, 1, &booster);
    if (status != 0) {
        xgbw_set_error("xgbw_train_eval: XGBoosterCreate failed: %s", XGBGetLastError());
        result = XGBW_ERROR_XGBOOST;
        goto cleanup_xgb;
    }

    /* Parse config and train */
    int n_estimators = 0;
    for (int i = 0; i < len_config; ++i) {
        if (config[i].key == NULL || config[i].value == NULL) continue;
        if (strcmp(config[i].key, "n_estimators") == 0) {
            n_estimators = atoi(config[i].value);
            continue;
        }
        XGBoosterSetParam(booster, config[i].key, config[i].value);
    }

    if (n_estimators < 1) {
        xgbw_set_error("xgbw_train_eval: n_estimators must be >= 1 (got %d)", n_estimators);
        result = XGBW_ERROR_INVALID_PARAM;
        goto cleanup_xgb;
    }

    for (int iter = 0; iter < n_estimators; ++iter) {
        status = XGBoosterUpdateOneIter(booster, iter, dtrain);
        if (status != 0) {
            xgbw_set_error("xgbw_train_eval: iteration %d failed: %s", iter, XGBGetLastError());
            result = XGBW_ERROR_XGBOOST;
            goto cleanup_xgb;
        }
    }
    
    /* Predict on test data */
    status = XGDMatrixCreateFromMat(x_test, (bst_ulong)rows_test, (bst_ulong)x_cols, -1.0f, &dtest);
    if (status != 0) {
        xgbw_set_error("xgbw_train_eval: XGDMatrixCreateFromMat (test) failed: %s", XGBGetLastError());
        result = XGBW_ERROR_XGBOOST;
        goto cleanup_xgb;
    }
    
    bst_ulong out_len = 0;
    const float* out_result = NULL;
    status = XGBoosterPredict(booster, dtest, 0, 0, 0, &out_len, &out_result);
    if (status != 0) {
        xgbw_set_error("xgbw_train_eval: XGBoosterPredict failed: %s", XGBGetLastError());
        result = XGBW_ERROR_XGBOOST;
        goto cleanup_xgb;
    }
    
    if (out_len != (bst_ulong)(rows_test * y_cols)) {
        xgbw_set_error("xgbw_train_eval: prediction size mismatch (expected %d, got %lu)", 
                       rows_test * y_cols, (unsigned long)out_len);
        result = XGBW_ERROR_SIZE_MISMATCH;
        goto cleanup_xgb;
    }
    
    memcpy(y_pred, out_result, out_len * sizeof(float));
    
    /* Calculate RMSE */
    calculate_rmse(y_pred, y_test, rows_test, y_cols, rmse_out);
    
    /* Save model in UBJSON format */
    bst_ulong buf_len = 0;
    const char* buf_data = NULL;
    status = XGBoosterSaveModelToBuffer(booster, "{\"format\": \"ubj\"}", &buf_len, &buf_data);
    if (status != 0) {
        xgbw_set_error("xgbw_train_eval: XGBoosterSaveModelToBuffer failed: %s", XGBGetLastError());
        result = XGBW_ERROR_XGBOOST;
        goto cleanup_xgb;
    }

    FILE* f = fopen(full_path, "wb");
    if (f == NULL) {
        xgbw_set_error("xgbw_train_eval: failed to open %s for writing", full_path);
        result = XGBW_ERROR_FILE_IO;
        goto cleanup_xgb;
    }

    size_t written = fwrite(buf_data, 1, buf_len, f);
    fclose(f);

    if (written != buf_len) {
        xgbw_set_error("xgbw_train_eval: failed to write model");
        result = XGBW_ERROR_FILE_IO;
        goto cleanup_xgb;
    }

cleanup_xgb:
    if (booster) XGBoosterFree(booster);
    if (dtrain) XGDMatrixFree(dtrain);
    if (dtest) XGDMatrixFree(dtest);
    
cleanup_buffers:
    free(x_train);
    free(y_train);
    free(x_test);
    free(y_test);
    free(y_pred);
    
    return result;
}

/* ===========================================================================
 * Public API: Inference
 * ===========================================================================*/

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
        xgbw_set_error("xgbw_predict: XGDMatrixCreateFromMat failed: %s", XGBGetLastError());
        return XGBW_ERROR_XGBOOST;
    }

    /* Create and load booster */
    status = XGBoosterCreate(NULL, 0, &booster);
    if (status != 0) {
        xgbw_set_error("xgbw_predict: XGBoosterCreate failed: %s", XGBGetLastError());
        result = XGBW_ERROR_XGBOOST;
        goto cleanup;
    }

    status = XGBoosterLoadModel(booster, inference_path);
    if (status != 0) {
        xgbw_set_error("xgbw_predict: failed to load model from %s: %s", inference_path, XGBGetLastError());
        result = XGBW_ERROR_FILE_IO;
        goto cleanup;
    }

    /* Make predictions */
    bst_ulong out_len = 0;
    const float* out_result = NULL;
    
    status = XGBoosterPredict(booster, dmatrix, 0, 0, 0, &out_len, &out_result);
    if (status != 0) {
        xgbw_set_error("xgbw_predict: XGBoosterPredict failed: %s", XGBGetLastError());
        result = XGBW_ERROR_XGBOOST;
        goto cleanup;
    }

    /* Validate output size */
    bst_ulong expected_len = (bst_ulong)(y_cols * rows);
    if (out_len != expected_len) {
        xgbw_set_error("xgbw_predict: size mismatch (expected %lu, got %lu)", 
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
