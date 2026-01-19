/**
 * @file test_xgbwrapper.c
 * @brief Test suite implementation for xgbwrapper library
 */

#include "test_xgbwrapper.h"

/* ===========================================================================
 * Test Configuration
 * ===========================================================================*/

/* Path for temporary model storage during tests */
static const char* TEST_MODEL_PATH = "/tmp/xgbwrapper_test_model.json";

/* Custom log callback for tests */
static void test_log_callback(int level, const char* msg) {
    const char* level_str[] = {"ERROR", "WARN", "INFO", "DEBUG"};
    if (level <= 2) {  /* Only show up to INFO in tests */
        printf("[%s] %s\n", level_str[level], msg);
    }
}

/* ===========================================================================
 * Test Utilities
 * ===========================================================================*/

void generate_simple_data_2cols(float* x, float* y, int rows, int x_cols) {
    const int y_cols = 2;
    
    /* Generate sequential features */
    for (int i = 0; i < rows; ++i) {
        for (int j = 0; j < x_cols; ++j) {
            x[i * x_cols + j] = (float)(i * x_cols + j);
        }
    }

    /* Compute targets: y[0] = sum(x), y[1] = -sum(x) */
    for (int i = 0; i < rows; ++i) {
        float sum = 0.0f;
        for (int k = 0; k < x_cols; ++k) {
            sum += x[i * x_cols + k];
        }
        y[i * y_cols + 0] = sum;
        y[i * y_cols + 1] = -sum;
    }
}

void print_data(float* x, float* y, int rows, int x_cols, int y_cols) {
    printf("Features (x):\n");
    for (int i = 0; i < rows; ++i) {
        printf("  [%d]: ", i);
        for (int j = 0; j < x_cols; ++j) {
            printf("%.4f ", x[i * x_cols + j]);
        }
        printf("\n");
    }
    
    printf("Targets (y):\n");
    for (int i = 0; i < rows; ++i) {
        printf("  [%d]: ", i);
        for (int j = 0; j < y_cols; ++j) {
            printf("%.4f ", y[i * y_cols + j]);
        }
        printf("\n");
    }
}

/* ===========================================================================
 * Test Implementations
 * ===========================================================================*/

void test_shuffle(void) {
    printf("=== Test: xgbw_shuffle ===\n");
    
    const int n = 10;
    int* array = (int*)malloc((size_t)n * sizeof(int));
    
    /* Test with new API */
    XGBWrapperStatus status = xgbw_shuffle(array, n);
    
    if (status != XGBW_SUCCESS) {
        printf("FAIL: xgbw_shuffle returned %s\n", xgbw_status_string(status));
        free(array);
        return;
    }
    
    printf("Shuffled: ");
    for (int i = 0; i < n; ++i) {
        printf("%d ", array[i]);
    }
    printf("\n");
    
    /* Verify all elements are present (permutation check) */
    int sum = 0;
    for (int i = 0; i < n; ++i) {
        sum += array[i];
    }
    int expected_sum = n * (n - 1) / 2;
    
    if (sum == expected_sum) {
        printf("PASS: All elements preserved (sum = %d)\n", sum);
    } else {
        printf("FAIL: Element sum mismatch (got %d, expected %d)\n", sum, expected_sum);
    }
    
    /* Test error handling */
    status = xgbw_shuffle(NULL, n);
    if (status == XGBW_ERROR_INVALID_PARAM) {
        printf("PASS: NULL array correctly rejected\n");
    } else {
        printf("FAIL: NULL array should return XGBW_ERROR_INVALID_PARAM\n");
    }
    
    free(array);
    printf("\n");
}

void test_split_data(void) {
    printf("=== Test: xgbw_split_data ===\n");
    
    const int rows = 10;
    const int x_cols = 2;
    const int y_cols = 1;
    const int rows_train = 8;
    const int rows_test = rows - rows_train;
    
    /* Allocate data */
    float* x = (float*)malloc((size_t)(rows * x_cols) * sizeof(float));
    float* y = (float*)malloc((size_t)(rows * y_cols) * sizeof(float));
    
    /* Initialize with known values */
    for (int i = 0; i < rows; ++i) {
        for (int j = 0; j < x_cols; ++j) {
            x[i * x_cols + j] = (float)(i * x_cols + j);
        }
        y[i] = (float)i;
    }
    
    /* Allocate split arrays */
    float* x_train = (float*)malloc((size_t)(rows_train * x_cols) * sizeof(float));
    float* y_train = (float*)malloc((size_t)(rows_train * y_cols) * sizeof(float));
    float* x_test = (float*)malloc((size_t)(rows_test * x_cols) * sizeof(float));
    float* y_test = (float*)malloc((size_t)(rows_test * y_cols) * sizeof(float));
    
    /* Perform split using new API */
    XGBWrapperStatus status = xgbw_split_data(x, y, x_train, y_train, x_test, y_test, 
                                               x_cols, y_cols, rows, rows_train);
    
    if (status != XGBW_SUCCESS) {
        printf("FAIL: xgbw_split_data returned %s: %s\n", 
               xgbw_status_string(status), xgbw_get_last_error());
    } else {
        printf("Training set: %d samples, Test set: %d samples\n", rows_train, rows_test);
        printf("PASS: Data split completed successfully\n");
    }
    
    /* Test error handling */
    status = xgbw_split_data(NULL, y, x_train, y_train, x_test, y_test,
                             x_cols, y_cols, rows, rows_train);
    if (status == XGBW_ERROR_INVALID_PARAM) {
        printf("PASS: NULL input correctly rejected\n");
    }
    
    /* Cleanup */
    free(x); free(y);
    free(x_train); free(y_train);
    free(x_test); free(y_test);
    printf("\n");
}

void test_generate_data(void) {
    printf("=== Test: xgbw_generate_test_data ===\n");
    
    const int rows = 5;
    const int x_cols = 3;
    const int y_cols = 2;
    
    float* x = (float*)malloc((size_t)(rows * x_cols) * sizeof(float));
    float* y = (float*)malloc((size_t)(rows * y_cols) * sizeof(float));
    
    XGBWrapperStatus status = xgbw_generate_test_data(x, y, rows, x_cols);
    
    if (status != XGBW_SUCCESS) {
        printf("FAIL: xgbw_generate_test_data returned %s\n", xgbw_status_string(status));
        free(x); free(y);
        return;
    }
    
    print_data(x, y, rows, x_cols, y_cols);
    
    /* Verify y[0] = sum(x) for first row */
    float sum = 0.0f;
    for (int j = 0; j < x_cols; ++j) {
        sum += x[j];
    }
    
    if (fabsf(y[0] - sum) < 1e-5f) {
        printf("PASS: y[0] = sum(x) verified\n");
    } else {
        printf("FAIL: y[0] mismatch (got %.4f, expected %.4f)\n", y[0], sum);
    }
    
    free(x);
    free(y);
    printf("\n");
}

void test_generate_simple_data(void) {
    printf("=== Test: generate_simple_data_2cols (deterministic) ===\n");
    
    const int rows = 5;
    const int x_cols = 2;
    const int y_cols = 2;
    
    float* x = (float*)malloc((size_t)(rows * x_cols) * sizeof(float));
    float* y = (float*)malloc((size_t)(rows * y_cols) * sizeof(float));
    
    generate_simple_data_2cols(x, y, rows, x_cols);
    print_data(x, y, rows, x_cols, y_cols);
    
    printf("PASS: Deterministic data generated\n\n");
    
    free(x);
    free(y);
}

void test_xgboost(void) {
    printf("=== Test: XGBoost Training & Prediction (Production API) ===\n");
    
    /* Initialize the library */
    XGBWrapperStatus status = xgbw_init();
    if (status != XGBW_SUCCESS) {
        printf("FAIL: xgbw_init() returned %s: %s\n", 
               xgbw_status_string(status), xgbw_get_last_error());
        return;
    }
    printf("Library initialized successfully\n");
    
    /* Set custom log callback */
    xgbw_set_log_callback(test_log_callback);
    
    /* Configuration */
    const int rows = 10000;
    const int x_cols = 4;
    const int y_cols = 2;
    const float split_ratio = 0.8f;
    const int rows_train = (int)(rows * split_ratio);
    const int rows_test = rows - rows_train;

    printf("Dataset: %d samples, %d features, %d targets\n", rows, x_cols, y_cols);
    printf("Split: %d train, %d test\n", rows_train, rows_test);

    /* Allocate data */
    float* x = (float*)malloc((size_t)(rows * x_cols) * sizeof(float));
    float* y = (float*)malloc((size_t)(rows * y_cols) * sizeof(float));
    float* x_train = (float*)malloc((size_t)(rows_train * x_cols) * sizeof(float));
    float* y_train = (float*)malloc((size_t)(rows_train * y_cols) * sizeof(float));
    float* x_test = (float*)malloc((size_t)(rows_test * x_cols) * sizeof(float));
    float* y_test = (float*)malloc((size_t)(rows_test * y_cols) * sizeof(float));
    float* y_pred = NULL;
    float* rmse = NULL;
    
    if (!x || !y || !x_train || !y_train || !x_test || !y_test) {
        printf("FAIL: Memory allocation failed\n");
        goto cleanup;
    }
    
    /* Generate test data */
    status = xgbw_generate_test_data(x, y, rows, x_cols);
    if (status != XGBW_SUCCESS) {
        printf("FAIL: xgbw_generate_test_data returned %s: %s\n",
               xgbw_status_string(status), xgbw_get_last_error());
        goto cleanup;
    }
    printf("Test data generated successfully\n");

    /* Split data */
    status = xgbw_split_data(x, y, x_train, y_train, x_test, y_test, 
                             x_cols, y_cols, rows, rows_train);
    if (status != XGBW_SUCCESS) {
        printf("FAIL: xgbw_split_data returned %s: %s\n",
               xgbw_status_string(status), xgbw_get_last_error());
        goto cleanup;
    }
    printf("Data split successfully\n");
    
    /* Free original data - no longer needed */
    free(x); x = NULL;
    free(y); y = NULL;

    /* XGBoost configuration */
    KVPair config[] = {
        {"booster", "gbtree"},
        {"objective", "reg:squarederror"},
        {"eval_metric", "rmse"},
        {"nthread", "4"},
        {"max_depth", "6"},
        {"learning_rate", "0.1"},
        {"subsample", "0.8"},
        {"colsample_bytree", "0.8"},
        {"reg_alpha", "0.0"},
        {"reg_lambda", "1.0"},
        {"n_estimators", "100"},
        {"verbosity", "0"}
    };
    int len_config = sizeof(config) / sizeof(config[0]);

    /* Train model using production API */
    printf("Training model...\n");
    status = xgbw_train(x_train, y_train, rows_train, x_cols, y_cols, 
                        config, len_config, TEST_MODEL_PATH);
    if (status != XGBW_SUCCESS) {
        printf("FAIL: xgbw_train returned %s: %s\n",
               xgbw_status_string(status), xgbw_get_last_error());
        goto cleanup;
    }
    printf("Model saved to: %s\n", TEST_MODEL_PATH);
    
    /* Free training data */
    free(x_train); x_train = NULL;
    free(y_train); y_train = NULL;

    /* Make predictions using production API */
    printf("Making predictions...\n");
    y_pred = (float*)malloc((size_t)(rows_test * y_cols) * sizeof(float));
    if (!y_pred) {
        printf("FAIL: Memory allocation for predictions failed\n");
        goto cleanup;
    }
    
    status = xgbw_predict(x_test, rows_test, x_cols, y_cols, TEST_MODEL_PATH, y_pred);
    if (status != XGBW_SUCCESS) {
        printf("FAIL: xgbw_predict returned %s: %s\n",
               xgbw_status_string(status), xgbw_get_last_error());
        goto cleanup;
    }
    printf("Predictions completed successfully\n");
    
    free(x_test); x_test = NULL;

    /* Calculate RMSE */
    rmse = (float*)malloc((size_t)y_cols * sizeof(float));
    if (!rmse) {
        printf("FAIL: Memory allocation for RMSE failed\n");
        goto cleanup;
    }
    
    status = xgbw_calculate_rmse(y_pred, y_test, rows_test, y_cols, rmse);
    if (status != XGBW_SUCCESS) {
        printf("FAIL: xgbw_calculate_rmse returned %s: %s\n",
               xgbw_status_string(status), xgbw_get_last_error());
        goto cleanup;
    }
    
    printf("RMSE results:\n");
    for (int j = 0; j < y_cols; ++j) {
        printf("  Target %d: %.6f\n", j, rmse[j]);
    }

    /* Evaluate results */
    int passed = 1;
    for (int j = 0; j < y_cols; ++j) {
        if (rmse[j] > 1.0f) {  /* Reasonable threshold for this synthetic data */
            printf("WARNING: RMSE[%d] = %.4f exceeds threshold\n", j, rmse[j]);
            passed = 0;
        }
    }
    
    if (passed) {
        printf("PASS: Model trained and predictions within acceptable error\n");
    } else {
        printf("FAIL: Prediction error too high\n");
    }

cleanup:
    /* Cleanup all resources */
    free(x);
    free(y);
    free(x_train);
    free(y_train);
    free(x_test);
    free(y_test);
    free(y_pred);
    free(rmse);
    
    /* Cleanup library */
    xgbw_cleanup();
    printf("Library cleanup completed\n\n");
}

/* ===========================================================================
 * Main Entry Point
 * ===========================================================================*/

int main(int argc, char* argv[]) {
    if (argc < 2) {
        fprintf(stderr, "Usage: %s <test_name>\n\n", argv[0]);
        fprintf(stderr, "Available tests:\n");
        fprintf(stderr, "  test_shuffle          - Test Fisher-Yates shuffle\n");
        fprintf(stderr, "  test_split_data       - Test train/test splitting\n");
        fprintf(stderr, "  test_generate_data    - Test random data generation\n");
        fprintf(stderr, "  test_generate_simple_data - Test deterministic data\n");
        fprintf(stderr, "  test_xgboost          - Full training/prediction test\n");
        return EXIT_FAILURE;
    }

    const char* test_name = argv[1];

    if (strcmp(test_name, "test_shuffle") == 0) {
        test_shuffle();
    } else if (strcmp(test_name, "test_split_data") == 0) {
        test_split_data();
    } else if (strcmp(test_name, "test_generate_data") == 0) {
        test_generate_data();
    } else if (strcmp(test_name, "test_generate_simple_data") == 0) {
        test_generate_simple_data();
    } else if (strcmp(test_name, "test_xgboost") == 0) {
        test_xgboost();
    } else {
        fprintf(stderr, "Unknown test: %s\n", test_name);
        return EXIT_FAILURE;
    }

    return EXIT_SUCCESS;
}
